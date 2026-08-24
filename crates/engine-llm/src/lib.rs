//! OpenAI-compatible streaming LLM client
#![deny(clippy::all)]

mod client;
mod sse;

pub use client::*;
pub use sse::*;
