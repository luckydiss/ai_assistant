use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Speaker {
    Interviewer,
    Candidate,
}

impl Speaker {
    pub(crate) fn lane(&self) -> u8 {
        match self {
            Speaker::Interviewer => 0,
            Speaker::Candidate => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transcript {
    pub speaker: Speaker,
    pub text: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: u64,
}

impl PartialOrd for Transcript {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transcript {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_time
            .cmp(&other.start_time)
            .then_with(|| self.speaker.lane().cmp(&other.speaker.lane()))
            .then_with(|| self.text.cmp(&other.text))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub speaker: Speaker,
    pub text: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    /// Сообщение введено пользователем вручную (не распознано из аудио).
    #[serde(default)]
    pub typed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialogue {
    pub turns: Vec<Turn>,
    pub summary: String,
    pub total_turns: usize,
}
