# Design: Dialogue Assembler

## 1. Dialogue Assembler Structure

**crates/engine-dialogue/Cargo.toml:**

```toml
[package]
name = "engine-dialogue"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
tokio.workspace = true
tokio-stream.workspace = true
chrono.workspace = true
```

**crates/engine-dialogue/src/lib.rs:**

```rust
//! Dialogue assembler with reorder, merge, dedup
#![deny(clippy::all)]

mod assembler;
mod error;
mod types;

pub use assembler::*;
pub use error::*;
pub use types::*;
```

## 2. Error Types

**crates/engine-dialogue/src/error.rs:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DialogueError {
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Summary generation failed: {0}")]
    SummaryFailed(String),
}
```

## 3. Types

**crates/engine-dialogue/src/types.rs:**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Speaker {
    Interviewer,
    Candidate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub speaker: Speaker,
    pub text: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub speaker: Speaker,
    pub text: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    pub turns: Vec<Turn>,
    pub summary: String,
    pub total_turns: usize,
}
```

## 4. Assembler

**crates/engine-dialogue/src/assembler.rs:**

```rust
use crate::{Dialogue, DialogueError, Speaker, Transcript, Turn};
use chrono::{DateTime, Duration, Utc};
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use tokio::sync::mpsc;

pub struct Assembler {
    buffer: BinaryHeap<Reverse<Transcript>>,
    turns: Vec<Turn>,
    summary: String,
    merge_threshold_ms: i64,
    dedup_threshold_secs: i64,
    summary_threshold: usize,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            buffer: BinaryHeap::new(),
            turns: Vec::new(),
            summary: String::new(),
            merge_threshold_ms: 500,
            dedup_threshold_secs: 2,
            summary_threshold: 16,
        }
    }
    
    pub async fn process_transcript(&mut self, transcript: Transcript) -> Result<Option<Turn>, DialogueError> {
        // Add to buffer
        self.buffer.push(Reverse(transcript));
        
        // Process buffer
        self.process_buffer().await
    }
    
    async fn process_buffer(&mut self) -> Result<Option<Turn>, DialogueError> {
        // Process all transcripts in buffer by timestamp order
        while let Some(Reverse(transcript)) = self.buffer.pop() {
            // Filter garbage
            if self.is_garbage(&transcript) {
                tracing::debug!("Filtered garbage: {}", transcript.text);
                continue;
            }
            
            // Check for duplicates
            if self.is_duplicate(&transcript) {
                tracing::debug!("Filtered duplicate: {}", transcript.text);
                continue;
            }
            
            // Try to merge with last turn
            if let Some(last_turn) = self.turns.last_mut() {
                if self.can_merge(last_turn, &transcript) {
                    last_turn.text.push(' ');
                    last_turn.text.push_str(&transcript.text);
                    last_turn.end_time = transcript.start_time + Duration::milliseconds(transcript.duration_ms as i64);
                    tracing::debug!("Merged with last turn");
                    continue;
                }
            }
            
            // Create new turn
            let turn = Turn {
                speaker: transcript.speaker,
                text: transcript.text,
                start_time: transcript.start_time,
                end_time: transcript.start_time + Duration::milliseconds(transcript.duration_ms as i64),
            };
            
            self.turns.push(turn.clone());
            tracing::info!("New turn: {:?}: {}", turn.speaker, turn.text);
            
            // Check if summary needed
            if self.turns.len() >= self.summary_threshold {
                self.generate_summary().await?;
            }
            
            return Ok(Some(turn));
        }
        
        Ok(None)
    }
    
    fn is_garbage(&self, transcript: &Transcript) -> bool {
        let text = transcript.text.trim().to_lowercase();
        
        // Too short
        if text.split_whitespace().count() < 2 {
            return true;
        }
        
        // Filler words
        let fillers = ["ок", "okay", "хорошо", "ага", "угу", "да", "спасибо"];
        if fillers.contains(&text.as_str()) {
            return true;
        }
        
        false
    }
    
    fn is_duplicate(&self, transcript: &Transcript) -> bool {
        if self.turns.is_empty() {
            return false;
        }
        
        let last = self.turns.last().unwrap();
        
        // Same speaker
        if last.speaker != transcript.speaker {
            return false;
        }
        
        // Exact text match
        if last.text.trim().to_lowercase() == transcript.text.trim().to_lowercase() {
            // Within time threshold
            let time_diff = transcript.start_time.signed_duration_since(last.end_time);
            if time_diff.num_seconds() <= self.dedup_threshold_secs {
                return true;
            }
        }
        
        false
    }
    
    fn can_merge(&self, last_turn: &Turn, transcript: &Transcript) -> bool {
        // Same speaker
        if last_turn.speaker != transcript.speaker {
            return false;
        }
        
        // Short pause
        let pause_ms = transcript.start_time.signed_duration_since(last_turn.end_time).num_milliseconds();
        if pause_ms <= self.merge_threshold_ms && pause_ms >= 0 {
            return true;
        }
        
        false
    }
    
    async fn generate_summary(&mut self) -> Result<(), DialogueError> {
        if self.turns.len() < 4 {
            return Ok(());
        }
        
        // Take first 4 turns
        let to_summarize: Vec<_> = self.turns.drain(..4).collect();
        
        // Simple concatenation for now (can be replaced with LLM summarization later)
        let summary_text = to_summarize.iter()
            .map(|t| format!("{:?}: {}", t.speaker, t.text))
            .collect::<Vec<_>>()
            .join(" ");
        
        // Append to existing summary
        if self.summary.is_empty() {
            self.summary = summary_text;
        } else {
            self.summary.push_str(&format!(" {}", summary_text));
        }
        
        tracing::info!("Generated summary: {}", self.summary);
        
        Ok(())
    }
    
    pub fn get_dialogue(&self) -> Dialogue {
        Dialogue {
            turns: self.turns.clone(),
            summary: self.summary.clone(),
            total_turns: self.turns.len(),
        }
    }
    
    pub fn get_recent_turns(&self, count: usize) -> Vec<Turn> {
        let start = if self.turns.len() > count {
            self.turns.len() - count
        } else {
            0
        };
        
        self.turns[start..].to_vec()
    }
}
```

## 5. Usage Example

```rust
use engine_dialogue::{Assembler, Transcript, Speaker};
use chrono::Utc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut assembler = Assembler::new();
    
    // Simulate transcripts arriving out of order
    let t1 = Transcript {
        speaker: Speaker::Interviewer,
        text: "Hello".to_string(),
        start_time: Utc::now(),
        duration_ms: 500,
    };
    
    let t2 = Transcript {
        speaker: Speaker::Candidate,
        text: "Hi there".to_string(),
        start_time: t1.start_time - chrono::Duration::milliseconds(100), // Earlier!
        duration_ms: 600,
    };
    
    // Process t1 first (even though t2 is earlier)
    if let Some(turn) = assembler.process_transcript(t1).await? {
        println!("Turn 1: {:?}", turn);
    }
    
    // Process t2 (should be inserted before t1)
    if let Some(turn) = assembler.process_transcript(t2).await? {
        println!("Turn 2: {:?}", turn);
    }
    
    let dialogue = assembler.get_dialogue();
    println!("Dialogue: {} turns", dialogue.turns.len());
    
    Ok(())
}
```

## Рассмотрено и отклонено
- **LLM-based summarization:** отклонено для MVP, используем simple concatenation
- **Complex NLP для garbage filtering:** отклонено, используем rule-based подход
- **Persistent storage:** отклонено, диалог хранится в памяти (store будет в отдельном change)
