# Design: Speech-to-Text with Groq

## 1. STT Client Structure

**crates/engine-stt/Cargo.toml:**

```toml
[package]
name = "engine-stt"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
reqwest.workspace = true
tokio.workspace = true
tokio-stream.workspace = true
futures.workspace = true
uuid.workspace = true
chrono.workspace = true
hound = "3.5"
```

**crates/engine-stt/src/lib.rs:**

```rust
//! Speech-to-Text using Groq API
#![deny(clippy::all)]

mod client;
mod error;
mod processor;
mod queue;
mod types;

pub use client::*;
pub use error::*;
pub use processor::*;
pub use queue::*;
pub use types::*;
```

## 2. Error Types

**crates/engine-stt/src/error.rs:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SttError {
    #[error("Authentication failed")]
    Authentication,
    
    #[error("Queue full")]
    QueueFull,
    
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
    
    #[error("Circuit breaker open")]
    CircuitOpen,
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Audio encoding error: {0}")]
    Encoding(#[from] hound::Error),
    
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
    
    #[error("Timeout")]
    Timeout,
}
```

## 3. Types

**crates/engine-stt/src/types.rs:**

```rust
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct AudioSegment {
    pub id: uuid::Uuid,
    pub audio: Vec<f32>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Transcript {
    pub text: String,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

## 4. Groq Client

**crates/engine-stt/src/client.rs:**

```rust
use crate::{SttError, Transcript};
use reqwest::{Client, multipart};
use std::io::Cursor;

pub struct GroqClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl GroqClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            api_key,
            base_url: "https://api.groq.com/openai/v1".to_string(),
        }
    }
    
    pub async fn transcribe(&self, audio: &[f32]) -> Result<Transcript, SttError> {
        // Encode audio to WAV
        let wav_data = self.encode_wav(audio)?;
        
        // Create multipart form
        let form = multipart::Form::new()
            .text("model", "whisper-large-v3-turbo")
            .part("file", multipart::Part::bytes(wav_data)
                .file_name("audio.wav")
                .mime_str("audio/wav")?);
        
        // Send request
        let response = self.client
            .post(format!("{}/audio/transcriptions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;
        
        let status = response.status();
        
        if status == 401 {
            return Err(SttError::Authentication);
        }
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SttError::Api {
                status: status.as_u16(),
                message: error_text,
            });
        }
        
        let transcript: Transcript = response.json().await?;
        Ok(transcript)
    }
    
    fn encode_wav(&self, audio: &[f32]) -> Result<Vec<u8>, SttError> {
        let mut cursor = Cursor::new(Vec::new());
        
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
        
        for &sample in audio {
            let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(sample_i16)?;
        }
        
        writer.finalize()?;
        
        Ok(cursor.into_inner())
    }
}
```

## 5. Circuit Breaker

**crates/engine-stt/src/circuit.rs:**

```rust
use crate::CircuitState;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct CircuitBreaker {
    state: Mutex<CircuitState>,
    failure_count: AtomicU64,
    failure_threshold: u64,
    timeout_duration: Duration,
    last_failure: Mutex<Option<Instant>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, timeout_secs: u64) -> Self {
        Self {
            state: Mutex::new(CircuitState::Closed),
            failure_count: AtomicU64::new(0),
            failure_threshold,
            timeout_duration: Duration::from_secs(timeout_secs),
            last_failure: Mutex::new(None),
        }
    }
    
    pub async fn allow_request(&self) -> bool {
        let state = self.state.lock().await;
        
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let last_failure = self.last_failure.lock().await;
                if let Some(last) = *last_failure {
                    if last.elapsed() >= self.timeout_duration {
                        drop(state);
                        let mut state = self.state.lock().await;
                        *state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
    
    pub async fn record_success(&self) {
        let mut state = self.state.lock().await;
        if *state == CircuitState::HalfOpen {
            *state = CircuitState::Closed;
        }
        self.failure_count.store(0, Ordering::SeqCst);
    }
    
    pub async fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        
        if failures >= self.failure_threshold {
            let mut state = self.state.lock().await;
            *state = CircuitState::Open;
            let mut last_failure = self.last_failure.lock().await;
            *last_failure = Some(Instant::now());
        }
    }
}
```

## 6. Queue

**crates/engine-stt/src/queue.rs:**

```rust
use crate::{AudioSegment, GroqClient, SttError, Transcript, CircuitBreaker};
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::time::{sleep, Duration};

pub struct SttQueue {
    sender: mpsc::Sender<AudioSegment>,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl SttQueue {
    pub fn new(
        client: Arc<GroqClient>,
        max_concurrency: usize,
        max_queue_size: usize,
    ) -> (Self, mpsc::Receiver<(AudioSegment, Result<Transcript, SttError>)>) {
        let (input_sender, mut input_receiver) = mpsc::channel::<AudioSegment>(max_queue_size);
        let (output_sender, output_receiver) = mpsc::channel(max_queue_size);
        
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let circuit_breaker = Arc::new(CircuitBreaker::new(5, 30));
        
        let circuit_breaker_clone = circuit_breaker.clone();
        
        tokio::spawn(async move {
            while let Some(segment) = input_receiver.recv().await {
                let client = client.clone();
                let semaphore = semaphore.clone();
                let output_sender = output_sender.clone();
                let circuit_breaker = circuit_breaker_clone.clone();
                
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    let result = Self::transcribe_with_retries(
                        &client,
                        &segment,
                        &circuit_breaker,
                    ).await;
                    
                    let _ = output_sender.send((segment, result)).await;
                });
            }
        });
        
        (Self { sender: input_sender, circuit_breaker }, output_receiver)
    }
    
    pub async fn submit(&self, segment: AudioSegment) -> Result<(), SttError> {
        self.sender.send(segment).await.map_err(|_| SttError::QueueFull)
    }
    
    async fn transcribe_with_retries(
        client: &GroqClient,
        segment: &AudioSegment,
        circuit_breaker: &CircuitBreaker,
    ) -> Result<Transcript, SttError> {
        if !circuit_breaker.allow_request().await {
            return Err(SttError::CircuitOpen);
        }
        
        let mut delay_ms = 100;
        
        for attempt in 0..3 {
            match client.transcribe(&segment.audio).await {
                Ok(transcript) => {
                    circuit_breaker.record_success().await;
                    return Ok(transcript);
                }
                Err(e) => {
                    circuit_breaker.record_failure().await;
                    
                    // Don't retry on authentication errors
                    if matches!(e, SttError::Authentication) {
                        return Err(e);
                    }
                    
                    if attempt < 2 {
                        sleep(Duration::from_millis(delay_ms)).await;
                        delay_ms *= 2;
                    }
                }
            }
        }
        
        Err(SttError::MaxRetriesExceeded)
    }
}
```

## 7. Processor

**crates/engine-stt/src/processor.rs:**

```rust
use crate::{AudioSegment, GroqClient, SttQueue, SttError, Transcript};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct SttProcessor {
    queue: SttQueue,
}

impl SttProcessor {
    pub fn new(api_key: String, max_concurrency: usize) -> (Self, mpsc::Receiver<(AudioSegment, Result<Transcript, SttError>)>) {
        let client = Arc::new(GroqClient::new(api_key));
        let (queue, receiver) = SttQueue::new(client, max_concurrency, 100);
        
        (Self { queue }, receiver)
    }
    
    pub async fn process_segment(&self, segment: AudioSegment) -> Result<(), SttError> {
        self.queue.submit(segment).await
    }
}
```

## 8. Usage Example

```rust
use engine_stt::{SttProcessor, AudioSegment};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = "gsk_...";
    let (processor, mut receiver) = SttProcessor::new(api_key.to_string(), 3);
    
    // Spawn task to receive transcripts
    let handler = tokio::spawn(async move {
        while let Some((segment, result)) = receiver.recv().await {
            match result {
                Ok(transcript) => {
                    tracing::info!("Transcript for {}: {}", segment.id, transcript.text);
                }
                Err(e) => {
                    tracing::error!("Failed to transcribe {}: {}", segment.id, e);
                }
            }
        }
    });
    
    // Submit audio segments
    for i in 0..10 {
        let segment = AudioSegment {
            id: Uuid::new_v4(),
            audio: vec![0.0f32; 16000], // 1 second of silence
            duration_ms: 1000,
        };
        
        processor.process_segment(segment).await?;
    }
    
    handler.await?;
    
    Ok(())
}
```

## Рассмотрено и отклонено
- **Tower вместо ручной retry логики:** отклонено, слишком сложно для текущего scope
- **Async-trait для client:** отклонено, используем concrete types
- **Global client instance:** отклонено, используем Arc для sharing
