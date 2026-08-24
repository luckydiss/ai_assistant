use crate::{SttError, Transcript};
use reqwest::{multipart, Client};
use serde::Deserialize;
use std::io::Cursor;
use std::time::Duration;

#[derive(Deserialize)]
struct VerboseSegment {
    #[serde(default)]
    avg_logprob: f32,
    #[serde(default)]
    no_speech_prob: f32,
}

#[derive(Deserialize)]
struct VerboseJson {
    text: String,
    #[serde(default)]
    duration: f32,
    #[serde(default)]
    segments: Vec<VerboseSegment>,
}

pub struct GroqClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    language: String,
}

impl GroqClient {
    pub fn new(api_key: String) -> Result<Self, SttError> {
        Self::with_base_url(api_key, "https://api.groq.com/openai/v1".to_string())
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, SttError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(SttError::Http)?;

        Ok(Self {
            client,
            api_key,
            base_url,
            model: "whisper-large-v3-turbo".to_string(),
            language: "auto".to_string(),
        })
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    pub async fn transcribe(&self, audio: &[f32]) -> Result<Transcript, SttError> {
        let wav_data = self.encode_wav(audio)?;

        let mut form = multipart::Form::new()
            .text("model", self.model.clone())
            .text("response_format", "verbose_json")
            .part(
                "file",
                multipart::Part::bytes(wav_data)
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| {
                        SttError::Encoding(hound::Error::IoError(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            e.to_string(),
                        )))
                    })?,
            );
        if self.language != "auto" {
            form = form.text("language", self.language.clone());
        }

        let response = self
            .client
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

        let v: VerboseJson = response.json().await?;
        let avg_logprob = v
            .segments
            .iter()
            .map(|s| s.avg_logprob)
            .fold(0.0f32, f32::min);
        let no_speech_prob = v
            .segments
            .iter()
            .map(|s| s.no_speech_prob)
            .fold(0.0f32, f32::max);
        let confidence = (1.0 + avg_logprob).clamp(0.0, 1.0);
        Ok(Transcript {
            text: v.text,
            duration: v.duration,
            avg_logprob,
            no_speech_prob,
            confidence,
            duration_ms: (v.duration * 1000.0) as u64,
        })
    }

    fn encode_wav(&self, audio: &[f32]) -> Result<Vec<u8>, SttError> {
        encode_wav_impl(audio)
    }
}

fn encode_wav_impl(audio: &[f32]) -> Result<Vec<u8>, SttError> {
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
