# Design: Store + Replay

## 1. engine-store/Cargo.toml

```toml
[package]
name = "engine-store"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
uuid.workspace = true
rusqlite.workspace = true
hound = "3.5"
engine-dialogue = { path = "../engine-dialogue" }
engine-orchestrator = { path = "../engine-orchestrator" }
```

## 2. src/lib.rs

```rust
//! Local persistence: sqlite history + JSONL replay log
#![deny(clippy::all)]

mod error;
mod logger;
mod sqlite;

pub use error::*;
pub use logger::*;
pub use sqlite::*;
```

## 3. src/error.rs

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("wav: {0}")]
    Wav(#[from] hound::Error),
}
```

## 4. src/sqlite.rs

```rust
use crate::StoreError;
use rusqlite::{params, Connection};

pub struct SessionStore {
    conn: Connection,
}

pub struct LatencyStats {
    pub p50_ttft_ms: u64,
    pub p95_ttft_ms: u64,
    pub answered: u64,
    pub skipped: u64,
    pub errors: u64,
}

impl SessionStore {
    pub fn open(path: &str) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY, started_at TEXT NOT NULL,
                ended_at TEXT, config_json TEXT);
             CREATE TABLE IF NOT EXISTS turns (
                id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL,
                speaker TEXT NOT NULL, text TEXT NOT NULL,
                start_time TEXT NOT NULL, end_time TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS answers (
                id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL,
                trigger_kind TEXT NOT NULL, outcome TEXT NOT NULL,
                full_text TEXT, stt_latency_ms INTEGER, ttft_ms INTEGER,
                created_at TEXT NOT NULL);",
        )?;
        Ok(Self { conn })
    }

    pub fn start_session(&self, id: &str, config_json: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO sessions (id, started_at, config_json) VALUES (?1, ?2, ?3)",
            params![id, chrono::Utc::now().to_rfc3339(), config_json],
        )?;
        Ok(())
    }

    pub fn end_session(&self, id: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn insert_turn(&self, session_id: &str, speaker: &str, text: &str, start: &str, end: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO turns (session_id, speaker, text, start_time, end_time) VALUES (?1,?2,?3,?4,?5)",
            params![session_id, speaker, text, start, end],
        )?;
        Ok(())
    }

    pub fn insert_answer(&self, session_id: &str, trigger_kind: &str, outcome: &str, full_text: &str, stt_latency_ms: u64, ttft_ms: u64) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO answers (session_id, trigger_kind, outcome, full_text, stt_latency_ms, ttft_ms, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![session_id, trigger_kind, outcome, full_text, stt_latency_ms as i64, ttft_ms as i64, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn stats(&self, session_id: &str) -> Result<LatencyStats, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT ttft_ms, outcome FROM answers WHERE session_id = ?1 ORDER BY ttft_ms",
        )?;
        let rows: Vec<(i64, String)> = stmt.query_map(params![session_id], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?.collect::<Result<_, _>>()?;

        let ttfts: Vec<i64> = rows.iter().map(|r| r.0).collect();
        let q = |p: f64| -> u64 {
            if ttfts.is_empty() { return 0; }
            let idx = ((ttfts.len() as f64 - 1.0) * p).round() as usize;
            ttfts[idx] as u64
        };

        Ok(LatencyStats {
            p50_ttft_ms: q(0.5),
            p95_ttft_ms: q(0.95),
            answered: rows.iter().filter(|r| r.1 == "answered").count() as u64,
            skipped: rows.iter().filter(|r| r.1 == "skipped").count() as u64,
            errors: rows.iter().filter(|r| r.1 == "error").count() as u64,
        })
    }
}
```

## 5. src/logger.rs

```rust
use crate::StoreError;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReplayEvent {
    Segment { id: String, lane: String, duration_ms: u64 },
    Transcript { id: String, lane: String, text: String },
    Turn { speaker: String, text: String },
    Trigger { kind: String, focus: String },
    Answer { outcome: String, text: String, ttft_ms: u64 },
}

pub struct ReplayLogger {
    dir: PathBuf,
    events: File,
}

impl ReplayLogger {
    pub fn open(dir: PathBuf) -> Result<Self, StoreError> {
        std::fs::create_dir_all(dir.join("audio"))?;
        let events = OpenOptions::new()
            .create(true).append(true)
            .open(dir.join("events.jsonl"))?;
        Ok(Self { dir, events })
    }

    pub fn log(&mut self, ev: &ReplayEvent) -> Result<(), StoreError> {
        writeln!(self.events, "{}", serde_json::to_string(ev)?)?;
        Ok(())
    }

    pub fn save_segment_wav(&self, id: &str, audio: &[f32]) -> Result<(), StoreError> {
        let path = self.dir.join("audio").join(format!("{}.wav", id));
        let spec = hound::WavSpec { channels: 1, sample_rate: 16000, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
        let mut w = hound::WavWriter::create(path, spec)?;
        for &s in audio { w.write_sample((s * 32767.0).clamp(-32768.0, 32767.0) as i16)?; }
        w.finalize()?;
        Ok(())
    }

    pub fn read_events(dir: &Path) -> Result<Vec<ReplayEvent>, StoreError> {
        let f = File::open(dir.join("events.jsonl"))?;
        let mut out = Vec::new();
        for line in BufReader::new(f).lines() {
            let line = line?;
            if !line.trim().is_empty() { out.push(serde_json::from_str(&line)?); }
        }
        Ok(out)
    }
}
```

## 6. Чистая функция в engine-orchestrator

Добавить в `src/orchestrator.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerKind { Auto, Speculative, Manual }

/// Чистое решение о триггере. Используется в on_turn И в replay.
pub fn trigger_decision(turn: &Turn, min_words: usize, debounce_ms: u64) -> Option<(TriggerKind, u64)> {
    if turn.speaker != Speaker::Interviewer { return None; }
    let words = turn.text.split_whitespace().count();
    if words < min_words { return None; }
    if words > 14 && turn.text.contains('?') {
        return Some((TriggerKind::Speculative, 200));
    }
    Some((TriggerKind::Auto, debounce_ms))
}
```

Заменить инлайн-логику в `Inner::on_turn` на вызов `trigger_decision` (поведение идентично, спека 009 не меняется).

## 7. Assembler::with_params в engine-dialogue

```rust
pub fn with_params(merge_threshold_ms: i64, dedup_threshold_secs: i64, summary_threshold: usize) -> Self {
    Self {
        buffer: Default::default(),
        turns: Vec::new(),
        summary: String::new(),
        merge_threshold_ms,
        dedup_threshold_secs,
        summary_threshold,
    }
}
```

## 8. examples/replay.rs (engine-store)

```rust
use engine_dialogue::{Assembler, Speaker, Transcript};
use engine_orchestrator::trigger_decision;
use engine_store::{ReplayEvent, ReplayLogger};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let dir = Path::new(args.get(1).map(|s| s.as_str()).unwrap_or("session"));
    let merge_ms: i64 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(500);
    let min_words: usize = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(4);
    let debounce_ms: u64 = args.get(4).and_then(|v| v.parse().ok()).unwrap_or(600);

    let events = ReplayLogger::read_events(dir)?;

    // transcripts by id -> assembler в порядке событий
    let mut texts: std::collections::HashMap<String, String> = Default::default();
    let mut assembler = Assembler::with_params(merge_ms, 2, 16);
    let mut turns = Vec::new();

    for ev in &events {
        match ev {
            ReplayEvent::Transcript { id, lane, text } => {
                texts.insert(id.clone(), format!("{}|{}", lane, text));
            }
            ReplayEvent::Segment { id, .. } => {
                if let Some(val) = texts.get(id) {
                    let (lane, text) = val.split_once('|').unwrap();
                    let speaker = if lane == "I" { Speaker::Interviewer } else { Speaker::Candidate };
                    let tr = Transcript {
                        speaker, text: text.clone(),
                        start_time: chrono::Utc::now(), duration_ms: 1000,
                    };
                    // блокирующий вариант process: использовать tokio::main
                    if let Ok(Some(t)) = tokio_block_on(assembler.process_transcript(tr)) {
                        turns.push(t);
                    }
                }
            }
            _ => {}
        }
    }

    // симуляция триггеров с дебаунс-отменой
    let mut fire_at: Option<u64> = None;
    let mut fired = 0;
    let mut seq: u64 = 0;
    for t in &turns {
        seq += 1;
        if let Some((kind, delay)) = trigger_decision(t, min_words, debounce_ms) {
            let at = seq * 1000 + delay; // условное время: 1с на turn
            if let Some(prev) = fire_at {
                if at < prev { continue; } // первый отменён
            }
            fire_at = Some(at);
            fired += 1;
            println!("TRIGGER {:?} at {}ms: {}", kind, at, t.text);
        }
    }
    println!("turns={} fired={} (merge_ms={} min_words={} debounce={}ms)",
        turns.len(), fired, merge_ms, min_words, debounce_ms);
    Ok(())
}

fn tokio_block_on<F: std::future::Future>(f: F) -> Result<F::Output, ()> {
    Ok(tokio::runtime::Runtime::new().map_err(|_| ())?.block_on(f))
}
```

## 9. Wiring в main.rs (добавить в setup и таски)

```rust
// после создания компонентов:
let session_id = uuid::Uuid::new_v4().to_string();
let store = Arc::new(engine_store::SessionStore::open("history.db")?);
store.start_session(&session_id, &serde_json::to_string(&cfg)?)?;
let mut logger = engine_store::ReplayLogger::open(
    std::path::PathBuf::from(format!("sessions/{}", session_id)))?;

// в таске segmenters->stt, после получения сегмента:
//   logger.log(&ReplayEvent::Segment { id: id.to_string(), lane: "I".into(), duration_ms: s.duration_ms })?;
//   logger.save_segment_wav(&id.to_string(), &s.audio)?;
// в таске stt->assembler, после транскрипта:
//   logger.log(&ReplayEvent::Transcript { id: seg.id.to_string(), lane: ..., text: t.text.clone() })?;
//   store.insert_turn(...) при Some(turn); logger.log(Turn{...});
// в форвардере orchestrator: на первом Token считать ttft от Status("generating"),
//   на Done/Skipped: store.insert_answer(...); logger.log(Answer{...});
```

Точные места вставки помечены комментариями; НЕ менять логику каналов, только логирование.

## Рассмотрено и отклонено
- **Хранение сырого аудио до VAD:** отклонено — объём; тюнинг VAD отложен
- **SQLCipher:** отклонено — шифрование не требуется в MVP
