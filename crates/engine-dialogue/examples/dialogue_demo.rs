use chrono::{Duration, Utc};
use engine_dialogue::{Assembler, Speaker, Transcript};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut assembler = Assembler::new();

    let now = Utc::now();

    // Simulate transcripts arriving out of order: t2 is earlier than t1
    let t1 = Transcript {
        speaker: Speaker::Interviewer,
        text: "Hello there".to_string(),
        start_time: now,
        duration_ms: 500,
    };

    let t2 = Transcript {
        speaker: Speaker::Candidate,
        text: "Hi there".to_string(),
        start_time: now - Duration::milliseconds(100),
        duration_ms: 600,
    };

    if let Some(turn) = assembler.process_transcript(t1).await? {
        tracing::info!("Turn 1: {:?}: {}", turn.speaker, turn.text);
    }

    if let Some(turn) = assembler.process_transcript(t2).await? {
        tracing::info!("Turn 2: {:?}: {}", turn.speaker, turn.text);
    }

    let dialogue = assembler.get_dialogue();
    tracing::info!("Dialogue: {} turns", dialogue.turns.len());
    for (i, turn) in dialogue.turns.iter().enumerate() {
        tracing::info!("  {i}: {:?}: {}", turn.speaker, turn.text);
    }

    Ok(())
}
