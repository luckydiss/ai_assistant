use thiserror::Error;

#[derive(Debug, Error)]
pub enum TtsError {
    #[error("tts api_key is empty")]
    EmptyKey,
    #[error("cartesia session: {0}")]
    Session(String),
    #[error("playback: {0}")]
    Playback(String),
}

impl From<anyhow::Error> for TtsError {
    fn from(e: anyhow::Error) -> Self {
        TtsError::Session(e.to_string())
    }
}