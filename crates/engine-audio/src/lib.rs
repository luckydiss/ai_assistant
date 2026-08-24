//! Audio capture for Windows (WASAPI loopback + mic)
#![deny(clippy::all)]

mod capture;
mod error;
mod resampler;
mod stream;

pub use capture::*;
pub use error::*;
pub use resampler::*;
pub use stream::*;
