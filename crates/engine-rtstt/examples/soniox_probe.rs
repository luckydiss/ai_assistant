use engine_audio::AudioEngine;
use engine_rtstt::{RtEvent, SonioxConfig, CHUNK};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let secs: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let cfg = engine_config::Config::load("config.toml")?;
    let key = cfg.stt.soniox_api_key.clone();
    anyhow::ensure!(!key.is_empty(), "soniox_api_key is empty in config.toml");
    println!("Soniox key present, streaming {secs}s from mic...");

    let hints: Vec<String> = cfg
        .stt
        .language_hints
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut audio = AudioEngine::new();
    let mut mic_rx = audio.start_mic_capture(cfg.audio.mic_device.as_deref())?;

    let mut session = engine_rtstt::start(SonioxConfig {
        api_key: key,
        model: cfg.stt.model.clone(),
        language_hints: hints,
        utterance_idle_ms: cfg.stt.utterance_idle_ms,
        max_utterance_chars: cfg.stt.max_utterance_chars,
        ws_url: "wss://stt-rt.soniox.com/transcribe-websocket".into(),
    })?;

    let started = Instant::now();
    let mut bytes_sent = 0usize;
    let mut peak: i16 = 0;
    let mut buf: Vec<u8> = Vec::with_capacity(CHUNK * 2);

    loop {
        tokio::select! {
            b = mic_rx.recv() => {
                let Some(bytes) = b else { break };
                buf.extend_from_slice(&bytes);
                while buf.len() >= CHUNK {
                    let piece: Vec<u8> = buf.drain(..CHUNK).collect();
                    for s in piece.chunks_exact(2) {
                        let v = i16::from_le_bytes([s[0], s[1]]).abs();
                        if v > peak { peak = v; }
                    }
                    if session.audio_tx.send(piece).await.is_err() { break; }
                    bytes_sent += CHUNK;
                }
                if started.elapsed() >= Duration::from_secs(secs) {
                    break;
                }
            }
            ev = session.events.recv() => {
                let Some(ev) = ev else { break };
                match ev {
                    RtEvent::Partial { text, .. } => println!("[partial] {text}"),
                    RtEvent::Utterance { text, .. } => println!("[utterance] {text}"),
                    RtEvent::Error(e) => { eprintln!("[error] {e}"); break; }
                    RtEvent::Closed => { println!("[closed]"); break; }
                }
            }
        }
    }

    drop(session.audio_tx);
    println!("audio ended, waiting for finals...");
    let deadline = Instant::now() + Duration::from_secs(5);
    while let Ok(ev) =
        tokio::time::timeout_at(deadline.into(), session.events.recv()).await
    {
        match ev {
            Some(RtEvent::Partial { text, .. }) => println!("[partial] {text}"),
            Some(RtEvent::Utterance { text, .. }) => println!("[utterance] {text}"),
            Some(RtEvent::Error(e)) => { eprintln!("[error] {e}"); break; }
            Some(RtEvent::Closed) => { println!("[closed]"); break; }
            None => break,
        }
    }

    println!(
        "done: {} bytes sent in {:.2}s, peak amp = {peak}",
        bytes_sent,
        started.elapsed().as_secs_f32()
    );
    audio.stop();
    Ok(())
}