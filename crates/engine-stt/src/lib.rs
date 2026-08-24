//! Speech-to-Text using Groq or Deepgram API (batch fallback)
#![deny(clippy::all)]

mod circuit;
mod client;
mod deepgram;
mod error;
mod processor;
mod queue;
mod types;

pub use circuit::*;
pub use client::*;
pub use deepgram::*;
pub use error::*;
pub use processor::*;
pub use queue::*;
pub use types::*;

use std::sync::Arc;

pub enum SttClient {
    Groq(GroqClient),
    Deepgram(DeepgramClient),
}

impl SttClient {
    pub fn from_config(
        provider: &str,
        api_key: String,
        model: String,
        language: String,
    ) -> Result<Arc<Self>, SttError> {
        let client = match provider {
            "deepgram" => {
                SttClient::Deepgram(DeepgramClient::new(api_key, model)?.with_language(language))
            }
            _ => SttClient::Groq(
                GroqClient::new(api_key)?
                    .with_model(model)
                    .with_language(language),
            ),
        };
        Ok(Arc::new(client))
    }

    pub async fn transcribe(&self, audio: &[f32]) -> Result<Transcript, SttError> {
        match self {
            SttClient::Groq(c) => c.transcribe(audio).await,
            SttClient::Deepgram(c) => c.transcribe(audio).await,
        }
    }
}
