//! Voice Activity Detection using Silero VAD
#![deny(clippy::all)]

mod error;
mod processor;
mod segmenter;
mod types;

pub use error::*;
pub use processor::*;
pub use segmenter::*;
pub use types::*;
