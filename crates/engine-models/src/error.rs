#[derive(Debug, thiserror::Error)]
pub enum ModelsError {
    #[error("http {0}")]
    Http(u16),
    #[error("invalid response format")]
    InvalidResponse,
    #[error("model {0} not found in catalog")]
    UnknownModel(String),
    #[error(transparent)]
    Network(#[from] reqwest::Error),
}
