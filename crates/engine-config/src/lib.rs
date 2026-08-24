//! Configuration management with hot-reload
#![deny(clippy::all)]

mod config;
mod error;
mod watcher;

pub use config::*;
pub use error::*;
pub use watcher::*;
