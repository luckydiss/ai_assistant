use thiserror::Error;

#[derive(Debug, Error)]
pub enum VadError {
    #[error("Failed to load model: {0}")]
    ModelLoad(String),

    #[error("ONNX error: {0}")]
    Onnx(#[from] ort::Error),

    #[error("Invalid audio format: expected 16kHz mono f32")]
    InvalidFormat,

    #[error("Inference error: {0}")]
    Inference(String),
}
