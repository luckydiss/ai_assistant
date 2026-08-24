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
                    let speaker = if lane == "I" {
                        Speaker::Interviewer
                    } else {
                        Speaker::Candidate
                    };
                    let tr = Transcript {
                        speaker,
                        text: text.to_string(),
                        start_time: chrono::Utc::now(),
                        duration_ms: 1000,
                    };
                    if let Some(t) = tokio_block_on(assembler.process_transcript(tr))
                        .ok()
                        .and_then(Result::ok)
                        .flatten()
                    {
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
                if at < prev {
                    continue;
                } // первый отменён
            }
            fire_at = Some(at);
            fired += 1;
            println!("TRIGGER {:?} at {}ms: {}", kind, at, t.text);
        }
    }
    println!(
        "turns={} fired={} (merge_ms={} min_words={} debounce={}ms)",
        turns.len(),
        fired,
        merge_ms,
        min_words,
        debounce_ms
    );
    Ok(())
}

fn tokio_block_on<F: std::future::Future>(f: F) -> Result<F::Output, ()> {
    Ok(tokio::runtime::Runtime::new().map_err(|_| ())?.block_on(f))
}
