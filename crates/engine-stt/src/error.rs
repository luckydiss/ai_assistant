use thiserror::Error;

#[derive(Debug, Error)]
pub enum SttError {
    #[error("Authentication failed")]
    Authentication,

    #[error("Queue full")]
    QueueFull,

    #[error("Max retries exceeded: {last_error}")]
    MaxRetriesExceeded { last_error: String },

    #[error("Circuit breaker open")]
    CircuitOpen,

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Audio encoding error: {0}")]
    Encoding(#[from] hound::Error),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Timeout")]
    Timeout,
}
