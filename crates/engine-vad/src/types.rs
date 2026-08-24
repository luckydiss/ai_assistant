use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct VadResult {
    pub speech: bool,
    pub probability: f32,
}

#[derive(Debug, Clone)]
pub struct SpeechSegment {
    pub audio: Vec<f32>,
    pub start_time_ms: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VadState {
    Waiting,
    Recording,
    Paused,
    Sending,
}
