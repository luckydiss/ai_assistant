use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
}
