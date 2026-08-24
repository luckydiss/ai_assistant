use crate::StoreError;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ReplayEvent {
    Segment {
        id: String,
        lane: String,
        duration_ms: u64,
    },
    Transcript {
        id: String,
        lane: String,
        text: String,
    },
    Turn {
        speaker: String,
        text: String,
    },
    Trigger {
        kind: String,
        focus: String,
    },
    Answer {
        outcome: String,
        text: String,
        ttft_ms: u64,
    },
}

pub struct ReplayLogger {
    dir: PathBuf,
    events: File,
}

impl ReplayLogger {
    pub fn open(dir: PathBuf) -> Result<Self, StoreError> {
        std::fs::create_dir_all(dir.join("audio"))?;
        let events = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("events.jsonl"))?;
        Ok(Self { dir, events })
    }

    pub fn log(&mut self, ev: &ReplayEvent) -> Result<(), StoreError> {
        writeln!(self.events, "{}", serde_json::to_string(ev)?)?;
        Ok(())
    }

    pub fn save_segment_wav(&self, id: &str, audio: &[f32]) -> Result<(), StoreError> {
        let path = self.dir.join("audio").join(format!("{}.wav", id));
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec)?;
        for &s in audio {
            w.write_sample((s * 32767.0).clamp(-32768.0, 32767.0) as i16)?;
        }
        w.finalize()?;
        Ok(())
    }

    pub fn read_events(dir: &Path) -> Result<Vec<ReplayEvent>, StoreError> {
        let f = File::open(dir.join("events.jsonl"))?;
        let mut out = Vec::new();
        for line in BufReader::new(f).lines() {
            let line = line?;
            if !line.trim().is_empty() {
                out.push(serde_json::from_str(&line)?);
            }
        }
        Ok(out)
    }
}
