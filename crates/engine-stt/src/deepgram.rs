use crate::{SttError, Transcript};
use reqwest::Client;
use serde::Deserialize;
use std::io::Cursor;
use std::time::Duration;

#[derive(Deserialize)]
struct DeepgramAlternative {
    #[serde(default)]
    transcript: String,
    #[serde(default)]
    confidence: f32,
}

#[derive(Deserialize)]
struct DeepgramChannel {
    #[serde(default)]
    alternatives: Vec<DeepgramAlternative>,
}

#[derive(Default, Deserialize)]
struct DeepgramResults {
    #[serde(default)]
    channels: Vec<DeepgramChannel>,
}

#[derive(Default, Deserialize)]
struct DeepgramMetadata {
    #[serde(default)]
    duration: f32,
}

#[derive(Default, Deserialize)]
struct DeepgramResponse {
    #[serde(default)]
    metadata: DeepgramMetadata,
    #[serde(default)]
    results: DeepgramResults,
}

pub struct DeepgramClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    language: String,
}

impl DeepgramClient {
    pub fn new(api_key: String, model: String) -> Result<Self, SttError> {
        Self::with_base_url(
            api_key,
            model,
            "https://api.deepgram.com/v1".to_string(),
        )
    }

    pub fn with_base_url(api_key: String, model: String, base_url: String) -> Result<Self, SttError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(SttError::Http)?;

        Ok(Self {
            client,
            api_key,
            base_url,
            model,
            language: "auto".to_string(),
        })
    }

    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    pub async fn transcribe(&self, audio: &[f32]) -> Result<Transcript, SttError> {
        let wav_data = encode_wav(audio)?;

        // nova-3-general НЕ мультиязычная: она разбита на варианты по языкам,
        // и `auto` не выбирает русский вариант — русская речь распознаётся как
        // английский бред. Поэтому для auto подставляем ru (русскоязычный сценарий).
        let lang = if self.language == "auto" { "ru" } else { self.language.as_str() };

        let mut url = format!("{}/listen?model={}", self.base_url, self.model);
        if lang != "auto" {
            url.push_str(&format!("&language={}", lang));
        }
        url.push_str("&punctuate=true&smart_format=true");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", self.api_key))
            .header("Accept", "application/json")
            .header("Content-Type", "audio/wav")
            .body(wav_data)
            .send()
            .await?;

        let status = response.status();

        if status == 401 || status == 403 {
            return Err(SttError::Authentication);
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SttError::Api {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let v: DeepgramResponse = response.json().await?;

        let alt = v
            .results
            .channels
            .first()
            .and_then(|c| c.alternatives.first());

        let Some(alt) = alt else {
            return Err(SttError::Api {
                status: 200,
                message: "empty deepgram response".into(),
            });
        };

        let confidence = alt.confidence.clamp(0.0, 1.0);
        Ok(Transcript {
            text: alt.transcript.clone(),
            duration: v.metadata.duration,
            avg_logprob: -(1.0 - confidence),
            no_speech_prob: 0.0,
            confidence,
            duration_ms: (v.metadata.duration * 1000.0) as u64,
        })
    }
}

fn encode_wav(audio: &[f32]) -> Result<Vec<u8>, SttError> {
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