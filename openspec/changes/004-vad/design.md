# Design: Voice Activity Detection

## 1. VAD Processor Structure

**crates/engine-vad/Cargo.toml:**

```toml
[package]
name = "engine-vad"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
ort.workspace = true
tokio.workspace = true
tokio-stream.workspace = true
ndarray = "0.15"
```

**crates/engine-vad/src/lib.rs:**

```rust
//! Voice Activity Detection using Silero VAD
#![deny(clippy::all)]

mod error;
mod processor;
mod segmenter;
mod types;

pub use error::*;
pub use processor::*;
pub use segmenter::*;
pub use types::*;
```

## 2. Error Types

**crates/engine-vad/src/error.rs:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VadError {
    #[error("Failed to load model: {0}")]
    ModelLoad(String),
    
    #[error("ONNX error: {0}")]
    Onnx(#[from] ort::Error),
    
    #[error("Invalid audio format: expected 16kHz mono f32")]
    InvalidFormat,
    
    #[error("Inference error: {0}")]
    Inference(String),
}
```

## 3. Types

**crates/engine-vad/src/types.rs:**

```rust
#[derive(Debug, Clone)]
pub struct VadResult {
    pub speech: bool,
    pub probability: f32,
}

#[derive(Debug, Clone)]
pub struct SpeechSegment {
    pub audio: Vec<f32>,
    pub start_time_ms: u64,
    pub duration_ms: u64,
}
```

## 4. VAD Processor

**crates/engine-vad/src/processor.rs:**

```rust
use crate::{VadError, VadResult};
use ndarray::{ArrayD, ArrayView, IxDyn};
use ort::{GraphOptimizationLevel, Session};
use std::path::Path;

pub struct VadProcessor {
    session: Session,
    state: Vec<f32>,
    context_size: usize,
}

impl VadProcessor {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self, VadError> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(1)?
            .with_inter_threads(1)?
            .commit_from_file(model_path)
            .map_err(|e| VadError::ModelLoad(e.to_string()))?;
        
        // Silero VAD state: [2, 1, 128] = 256 floats
        let state = vec![0.0f32; 256];
        
        Ok(Self {
            session,
            state,
            context_size: 64, // Silero expects 512 samples at 16kHz
        })
    }
    
    pub fn process_chunk(&mut self, audio: &[f32]) -> Result<VadResult, VadError> {
        // Validate input
        if audio.is_empty() {
            return Ok(VadResult { speech: false, probability: 0.0 });
        }
        
        // Pad or truncate to 512 samples
        let mut input = vec![0.0f32; 512];
        let len = audio.len().min(512);
        input[..len].copy_from_slice(&audio[..len]);
        
        // Prepare input tensor: [1, 512]
        let input_tensor = ArrayView::from(&input)
            .into_shape(IxDyn(&[1, 512]))
            .map_err(|e| VadError::Inference(e.to_string()))?;
        
        // Prepare state tensor: [2, 1, 128]
        let state_tensor = ArrayView::from(&self.state)
            .into_shape(IxDyn(&[2, 1, 128]))
            .map_err(|e| VadError::Inference(e.to_string()))?;
        
        // Prepare sample_rate tensor: [1]
        let sample_rate = [16000i64];
        let sample_rate_tensor = ArrayView::from(&sample_rate)
            .into_shape(IxDyn(&[1]))
            .map_err(|e| VadError::Inference(e.to_string()))?;
        
        // Run inference
        let outputs = self.session.run(ort::inputs![
            "input" => input_tensor,
            "state" => state_tensor,
            "sr" => sample_rate_tensor
        ]?)?;
        
        // Extract output probability
        let output = outputs["output"].try_extract_tensor::<f32>()?;
        let probability = output[[0, 0]];
        
        // Update state
        let new_state = outputs["state"].try_extract_tensor::<f32>()?;
        self.state = new_state.iter().copied().collect();
        
        Ok(VadResult {
            speech: probability > 0.5,
            probability,
        })
    }
    
    pub fn reset(&mut self) {
        self.state.fill(0.0);
    }
}
```

## 5. Segmenter

**crates/engine-vad/src/segmenter.rs:**

```rust
use crate::{SpeechSegment, VadProcessor, VadResult};
use std::collections::VecDeque;
use tokio::sync::mpsc;

pub struct Segmenter {
    vad: VadProcessor,
    silence_ms: u64,
    max_segment_ms: u64,
    buffer: Vec<f32>,
    silence_duration_ms: u64,
    segment_start_ms: u64,
    current_time_ms: u64,
    sender: mpsc::Sender<SpeechSegment>,
}

impl Segmenter {
    pub fn new(
        vad: VadProcessor,
        silence_ms: u64,
        max_segment_ms: u64,
    ) -> (Self, mpsc::Receiver<SpeechSegment>) {
        let (sender, receiver) = mpsc::channel(32);
        
        let segmenter = Self {
            vad,
            silence_ms,
            max_segment_ms,
            buffer: Vec::with_capacity(112000), // 7 seconds at 16kHz
            silence_duration_ms: 0,
            segment_start_ms: 0,
            current_time_ms: 0,
            sender,
        };
        
        (segmenter, receiver)
    }
    
    pub async fn process_chunk(&mut self, audio: &[f32]) -> Result<(), crate::VadError> {
        let chunk_duration_ms = (audio.len() as u64 * 1000) / 16000;
        
        // Process in 512-sample chunks (32ms at 16kHz)
        for chunk in audio.chunks(512) {
            let vad_result = self.vad.process_chunk(chunk)?;
            
            if vad_result.speech {
                // Speech detected
                self.silence_duration_ms = 0;
                self.buffer.extend_from_slice(chunk);
            } else {
                // Silence detected
                self.silence_duration_ms += 32; // Each chunk is 32ms
                
                if !self.buffer.is_empty() {
                    self.buffer.extend_from_slice(chunk);
                }
            }
            
            self.current_time_ms += 32;
            
            // Check if we should emit a segment
            let should_emit = self.should_emit_segment(vad_result.speech);
            
            if should_emit && !self.buffer.is_empty() {
                self.emit_segment().await?;
            }
        }
        
        Ok(())
    }
    
    fn should_emit_segment(&self, is_speech: bool) -> bool {
        if self.buffer.is_empty() {
            return false;
        }
        
        let segment_duration_ms = (self.buffer.len() as u64 * 1000) / 16000;
        
        // Emit if silence >= threshold
        if !is_speech && self.silence_duration_ms >= self.silence_ms {
            return true;
        }
        
        // Emit if segment too long
        if segment_duration_ms >= self.max_segment_ms {
            return true;
        }
        
        false
    }
    
    async fn emit_segment(&mut self) -> Result<(), crate::VadError> {
        let duration_ms = (self.buffer.len() as u64 * 1000) / 16000;
        
        let segment = SpeechSegment {
            audio: std::mem::take(&mut self.buffer),
            start_time_ms: self.segment_start_ms,
            duration_ms,
        };
        
        self.sender.send(segment).await.map_err(|_| {
            crate::VadError::Inference("Channel closed".to_string())
        })?;
        
        self.segment_start_ms = self.current_time_ms;
        self.buffer.clear();
        self.silence_duration_ms = 0;
        
        Ok(())
    }
}
```

## 6. Usage Example

```rust
use engine_vad::{VadProcessor, Segmenter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let vad = VadProcessor::new("silero_vad.onnx")?;
    let (mut segmenter, mut receiver) = Segmenter::new(vad, 600, 7000);
    
    // Spawn task to receive segments
    let segment_handler = tokio::spawn(async move {
        while let Some(segment) = receiver.recv().await {
            tracing::info!(
                "Segment: start={}ms, duration={}ms, samples={}",
                segment.start_time_ms,
                segment.duration_ms,
                segment.audio.len()
            );
        }
    });
    
    // Simulate audio stream (replace with actual audio capture)
    let audio_chunks: Vec<Vec<f32>> = vec![/* audio data */];
    
    for chunk in audio_chunks {
        segmenter.process_chunk(&chunk).await?;
    }
    
    segment_handler.await?;
    
    Ok(())
}
```

## 7. Model Download

Silero VAD ONNX model нужно скачать отдельно:

```bash
# Download from Silero releases
curl -L -o silero_vad.onnx https://github.com/snakers4/silero-vad/raw/master/files/silero_vad.onnx
```

Model file: ~2MB, лицензия MIT.

## Рассмотрено и отклонено
- **WebRTC VAD вместо Silero:** отклонено, Silero имеет лучшее качество
- **Локальная компиляция ONNX:** отклонено, используем pre-built модель
- **Multi-threaded inference:** отклонено, один поток достаточно для real-time
