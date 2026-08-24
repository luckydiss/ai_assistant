//! Local persistence: sqlite history + JSONL replay log
#![deny(clippy::all)]

mod error;
mod logger;
mod sqlite;

pub use error::*;
pub use logger::*;
pub use sqlite::*;
