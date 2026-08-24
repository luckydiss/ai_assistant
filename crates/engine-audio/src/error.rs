use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No audio device available")]
    NoDevice,

    #[error("Host unavailable: {0}")]
    HostUnavailable(#[from] cpal::HostUnavailable),

    #[error("Default stream config error: {0}")]
    DefaultStreamConfig(#[from] cpal::DefaultStreamConfigError),

    #[error("Devices error: {0}")]
    Devices(#[from] cpal::DevicesError),

    #[error("Stream error: {0}")]
    Stream(#[from] cpal::BuildStreamError),

    #[error("Play error: {0}")]
    Play(#[from] cpal::PlayStreamError),

    #[error("Unsupported sample rate: {0}")]
    UnsupportedSampleRate(u32),

    #[error("Resampler error: {0}")]
    Resampler(String),
}
