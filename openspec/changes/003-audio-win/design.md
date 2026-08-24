# Design: Windows Audio Capture

## 1. Audio Engine Structure

**crates/engine-audio/Cargo.toml:**

```toml
[package]
name = "engine-audio"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
cpal.workspace = true
tokio.workspace = true
tokio-stream.workspace = true
rubato = "0.14"
```

**crates/engine-audio/src/lib.rs:**

```rust
//! Audio capture for Windows (WASAPI loopback + mic)
#![deny(clippy::all)]

mod capture;
mod error;
mod resampler;
mod stream;

pub use capture::*;
pub use error::*;
pub use stream::*;
```

## 2. Error Types

**crates/engine-audio/src/error.rs:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No audio device available")]
    NoDevice,
    
    #[error("Device error: {0}")]
    Device(#[from] cpal::DeviceUnavailable),
    
    #[error("Stream error: {0}")]
    Stream(#[from] cpal::BuildStreamError),
    
    #[error("Play error: {0}")]
    Play(#[from] cpal::PlayStreamError),
    
    #[error("Unsupported sample rate: {0}")]
    UnsupportedSampleRate(u32),
}
```

## 3. Resampler

**crates/engine-audio/src/resampler.rs:**

```rust
use rubato::{FftFixedInOut, Resampler};

pub struct AudioResampler {
    resampler: FftFixedInOut<f32>,
    input_buffer: Vec<Vec<f32>>,
    output_buffer: Vec<Vec<f32>>,
}

impl AudioResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Result<Self, crate::AudioError> {
        if input_rate == output_rate {
            return Err(crate::AudioError::UnsupportedSampleRate(input_rate));
        }
        
        let resampler = FftFixedInOut::<f32>::new(
            input_rate as usize,
            output_rate as usize,
            1024,
            1,
        )?;
        
        let input_buffer = resampler.input_buffer_allocate();
        let output_buffer = resampler.output_buffer_allocate();
        
        Ok(Self {
            resampler,
            input_buffer,
            output_buffer,
        })
    }
    
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if self.resampler.input_frames_next() == 0 {
            // No resampling needed or same rate
            return input.to_vec();
        }
        
        // Copy input to buffer
        for (i, &sample) in input.iter().enumerate() {
            if i < self.input_buffer[0].len() {
                self.input_buffer[0][i] = sample;
            }
        }
        
        // Process
        self.resampler.process_into_buffer(&self.input_buffer, &mut self.output_buffer, None)?;
        
        self.output_buffer[0].clone()
    }
}
```

## 4. Audio Capture

**crates/engine-audio/src/capture.rs:**

```rust
use crate::{AudioError, AudioEvent, AudioStream};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub struct AudioEngine {
    sender: broadcast::Sender<AudioEvent>,
    streams: Arc<Mutex<Vec<Stream>>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            sender,
            streams: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<AudioEvent> {
        self.sender.subscribe()
    }
    
    pub fn start_system_capture(&self) -> Result<(), AudioError> {
        let host = cpal::host_from_id(cpal::HostId::Wasapi)?;
        let device = host.default_output_device()
            .ok_or(AudioError::NoDevice)?;
        
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        
        tracing::info!("System audio: {}Hz, {} channels", sample_rate, channels);
        
        let sender = self.sender.clone();
        let mut resampler = if sample_rate != 16000 {
            Some(crate::resampler::AudioResampler::new(sample_rate, 16000)?)
        } else {
            None
        };
        
        let stream_config = StreamConfig {
            channels: 1, // mono
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        
        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono = Self::to_mono(data, channels);
                    let resampled = if let Some(ref mut r) = resampler {
                        r.process(&mono)
                    } else {
                        mono
                    };
                    let _ = sender.send(AudioEvent::SystemData(resampled));
                },
                |err| tracing::error!("System audio error: {}", err),
                None,
            )?,
            SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    let mono = Self::to_mono(&f32_data, channels);
                    let resampled = if let Some(ref mut r) = resampler {
                        r.process(&mono)
                    } else {
                        mono
                    };
                    let _ = sender.send(AudioEvent::SystemData(resampled));
                },
                |err| tracing::error!("System audio error: {}", err),
                None,
            )?,
            _ => return Err(AudioError::UnsupportedSampleRate(sample_rate)),
        };
        
        stream.play()?;
        self.streams.lock().unwrap().push(stream);
        
        Ok(())
    }
    
    pub fn start_mic_capture(&self) -> Result<(), AudioError> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or(AudioError::NoDevice)?;
        
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        
        tracing::info!("Mic audio: {}Hz, {} channels", sample_rate, channels);
        
        let sender = self.sender.clone();
        let mut resampler = if sample_rate != 16000 {
            Some(crate::resampler::AudioResampler::new(sample_rate, 16000)?)
        } else {
            None
        };
        
        let stream_config = StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        
        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono = Self::to_mono(data, channels);
                    let resampled = if let Some(ref mut r) = resampler {
                        r.process(&mono)
                    } else {
                        mono
                    };
                    let _ = sender.send(AudioEvent::MicData(resampled));
                },
                |err| tracing::error!("Mic audio error: {}", err),
                None,
            )?,
            SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    let mono = Self::to_mono(&f32_data, channels);
                    let resampled = if let Some(ref mut r) = resampler {
                        r.process(&mono)
                    } else {
                        mono
                    };
                    let _ = sender.send(AudioEvent::MicData(resampled));
                },
                |err| tracing::error!("Mic audio error: {}", err),
                None,
            )?,
            _ => return Err(AudioError::UnsupportedSampleRate(sample_rate)),
        };
        
        stream.play()?;
        self.streams.lock().unwrap().push(stream);
        
        Ok(())
    }
    
    fn to_mono(data: &[f32], channels: u16) -> Vec<f32> {
        if channels == 1 {
            return data.to_vec();
        }
        
        let mut mono = Vec::with_capacity(data.len() / channels as usize);
        for chunk in data.chunks(channels as usize) {
            let sum: f32 = chunk.iter().sum();
            mono.push(sum / channels as f32);
        }
        mono
    }
}
```

## 5. Stream Types

**crates/engine-audio/src/stream.rs:**

```rust
#[derive(Debug, Clone)]
pub enum AudioEvent {
    SystemData(Vec<f32>),
    MicData(Vec<f32>),
}

pub type AudioStream = tokio::sync::broadcast::Receiver<AudioEvent>;
```

## 6. Usage Example

```rust
use engine_audio::AudioEngine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = AudioEngine::new();
    
    engine.start_system_capture()?;
    engine.start_mic_capture()?;
    
    let mut receiver = engine.subscribe();
    
    loop {
        match receiver.recv().await? {
            AudioEvent::SystemData(data) => {
                tracing::debug!("System: {} samples", data.len());
            }
            AudioEvent::MicData(data) => {
                tracing::debug!("Mic: {} samples", data.len());
            }
        }
    }
}
```

## Рассмотрено и отклонено
- **Symphonia вместо cpal:** отклонено, cpal имеет прямой WASAPI access
- **Multi-threaded audio processing:** отклонено, используем broadcast channel для разделения
- **WASAPI exclusive mode:** отклонено, shared mode проще и достаточно
