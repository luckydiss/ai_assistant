use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::mpsc;
use tokio::task::AbortHandle;
use tokio_tungstenite::tungstenite::Message;

pub const CHUNK: usize = 3840; // 120 мс @16kHz s16le mono

#[derive(Debug, Clone)]
pub struct SonioxConfig {
    pub api_key: String,
    pub model: String,
    pub language_hints: Vec<String>,
    pub utterance_idle_ms: u64,
    pub max_utterance_chars: usize,
    pub ws_url: String,
}

impl SonioxConfig {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            language_hints: vec!["ru".into()],
            utterance_idle_ms: 700,
            max_utterance_chars: 600,
            ws_url: "wss://stt-rt.soniox.com/transcribe-websocket".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RtEvent {
    Partial { text: String, utt_id: String },
    Utterance { text: String, utt_id: String },
    Error(String),
    Closed,
}

pub struct RtSession {
    pub audio_tx: mpsc::Sender<Vec<u8>>,
    pub events: mpsc::Receiver<RtEvent>,
    pub handle: AbortHandle,
}

pub fn start(cfg: SonioxConfig) -> anyhow::Result<RtSession> {
    anyhow::ensure!(!cfg.api_key.is_empty(), "soniox api_key empty");
    let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(256);
    let (ev_tx, events) = mpsc::channel::<RtEvent>(256);

    let task = tokio::spawn(async move {
        let url = cfg.ws_url.clone();
        let Ok((mut ws, _)) = tokio_tungstenite::connect_async(url).await else {
            let _ = ev_tx.send(RtEvent::Error("connect failed".into())).await;
            return;
        };
        tracing::info!("soniox ws connected");

        let idle_ms = cfg.utterance_idle_ms;
        let max_chars = cfg.max_utterance_chars;
        let init = json!({
            "api_key": cfg.api_key,
            "model": cfg.model,
            "audio_format": "pcm_s16le",
            "sample_rate": 16000,
            "num_channels": 1,
            "language_hints": cfg.language_hints,
            "enable_endpoint_detection": false,
        });
        if ws.send(Message::Text(init.to_string())).await.is_err() {
            return;
        }
        tracing::info!("soniox config sent");

        let mut finals: Vec<String> = Vec::new();
        let mut last_final_at: Option<std::time::Instant> = None;
        let mut utt_counter: u64 = 0;
        let mut utt_id = format!("u{}", utt_counter);
        let mut tick = tokio::time::interval(std::time::Duration::from_millis(200));

        loop {
            tokio::select! {
                a = audio_rx.recv() => match a {
                    Some(bytes) => {
                        if ws.send(Message::Binary(bytes)).await.is_err() {
                            break;
                        }
                    }
                    None => {
                        let _ = ws.send(Message::Text("".into())).await;
                        break;
                    }
                },
                m = ws.next() => match m {
                    Some(Ok(Message::Text(t))) => {
                        let v: serde_json::Value = match serde_json::from_str(&t) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        if let Some(code) = v["error_code"].as_str() {
                            let _ = ev_tx
                                .send(RtEvent::Error(format!(
                                    "{code}: {}",
                                    v["error_message"]
                                )))
                                .await;
                            break;
                        }
                        let mut nonfinal = String::new();
                        let mut got_final = false;
                        for tok in v["tokens"].as_array().into_iter().flatten() {
                            let raw = tok["text"].as_str().unwrap_or("");
                            let s = clean_token(raw);
                            if tok["is_final"].as_bool().unwrap_or(false) {
                                finals.push(s);
                                got_final = true;
                            } else {
                                nonfinal.push_str(&s);
                            }
                        }
                        if got_final {
                            last_final_at = Some(std::time::Instant::now());
                        }
                        let partial = render(&finals, &nonfinal);
                        if !partial.is_empty() {
                            tracing::debug!("soniox partial: {partial}");
                            let _ = ev_tx
                                .send(RtEvent::Partial {
                                    text: partial,
                                    utt_id: utt_id.clone(),
                                })
                                .await;
                        }
                        if v["finished"].as_bool().unwrap_or(false) {
                            flush(&mut finals, &ev_tx, &utt_id).await;
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => {
                        flush(&mut finals, &ev_tx, &utt_id).await;
                        break;
                    }
                    _ => {}
                },
                _ = tick.tick() => {
                    if let Some(t) = last_final_at {
                        if !finals.is_empty() {
                            let now = std::time::Instant::now();
                            let idle = now.duration_since(t).as_millis() as u64;
                            let chars: usize =
                                finals.iter().map(|s| s.chars().count()).sum();

                            let close_pause = idle >= idle_ms;
                            let close_len = chars >= max_chars && idle >= 300;

                            if close_pause || close_len {
                                flush(&mut finals, &ev_tx, &utt_id).await;
                                utt_counter += 1;
                                utt_id = format!("u{}", utt_counter);
                                last_final_at = None;
                            }
                        }
                    }
                },
            }
        }
        let _ = ev_tx.send(RtEvent::Closed).await;
    });

    Ok(RtSession {
        audio_tx,
        events,
        handle: task.abort_handle(),
    })
}

/// Склеивает subword-токены как есть (пробел кодируется отдельным токеном),
/// затем схлопывает \n и множественные пробелы в один.
fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !last_ws && !out.is_empty() {
                out.push(' ');
                last_ws = true;
            }
        } else {
            out.push(c);
            last_ws = false;
        }
    }
    out.trim().to_string()
}

pub fn render(finals: &[String], nonfinal: &str) -> String {
    let raw: String = finals.iter().cloned().collect::<String>() + nonfinal;
    normalize(&raw)
}

/// Убирает служебные теги Soniox (<end>, <sil>, <noise> и пр.) из текста токена,
/// сохраняя значимые пробелы между словами.
fn clean_token(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

async fn flush(finals: &mut Vec<String>, ev: &mpsc::Sender<RtEvent>, utt_id: &str) {
    let raw: String = finals.drain(..).collect();
    let text = normalize(&raw);
    if !text.is_empty() {
        tracing::info!("soniox utterance: {text}");
        let _ = ev
            .send(RtEvent::Utterance {
                text,
                utt_id: utt_id.to_string(),
            })
            .await;
    }
}