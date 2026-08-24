use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::mpsc;
use tokio::task::AbortHandle;
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest, http::HeaderValue, Message,
};

pub struct CartesiaConfig {
    pub api_key: String,
    pub model_id: String,
    pub voice_id: String,
    pub sample_rate: u32,
}

pub enum TtsCmd {
    Text(String),
    Flush,
}

pub enum TtsOut {
    Pcm(Vec<f32>),
    Done,
    Error(String),
}

pub struct TtsSession {
    pub cmd: mpsc::Sender<TtsCmd>,
    pub out: mpsc::Receiver<TtsOut>,
    pub handle: AbortHandle,
}

fn decode_pcm(data: &[u8]) -> Vec<f32> {
    data.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

pub fn start_session(cfg: CartesiaConfig) -> anyhow::Result<TtsSession> {
    start_session_at(cfg, "wss://api.cartesia.ai/tts/websocket".to_string())
}

/// Starts a session against a custom endpoint (tests use the mock server).
pub fn start_session_at(cfg: CartesiaConfig, url: String) -> anyhow::Result<TtsSession> {
    anyhow::ensure!(!cfg.api_key.is_empty(), "tts api_key is empty");
    let (cmd, mut cmd_rx) = mpsc::channel::<TtsCmd>(64);
    let (out_tx, out) = mpsc::channel::<TtsOut>(64);

    let task = tokio::spawn(async move {
        let mut req = match url.into_client_request() {
            Ok(r) => r,
            Err(e) => {
                let _ = out_tx.send(TtsOut::Error(e.to_string())).await;
                return;
            }
        };
        req.headers_mut().insert(
            "X-API-Key",
            cfg.api_key.parse().unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        req.headers_mut().insert(
            "Cartesia-Version",
            "2024-06-10"
                .parse()
                .unwrap_or_else(|_| HeaderValue::from_static("")),
        );

        let mut ws = match tokio_tungstenite::connect_async(req).await {
            Ok((ws, _)) => ws,
            Err(e) => {
                let _ = out_tx.send(TtsOut::Error(e.to_string())).await;
                return;
            }
        };
        let ctx_id = uuid::Uuid::new_v4().to_string();

        let build = |transcript: &str, cont: bool| {
            json!({
                "model_id": cfg.model_id,
                "transcript": transcript,
                "continue": cont,
                "voice": { "mode": "id", "id": cfg.voice_id },
                "output_format": { "container": "raw", "encoding": "pcm_f32le", "sample_rate": cfg.sample_rate },
                "context_id": ctx_id,
            })
            .to_string()
        };

        loop {
            tokio::select! {
                cmd = cmd_rx.recv() => match cmd {
                    Some(TtsCmd::Text(t)) => {
                        let msg = build(&t, true);
                        if ws.send(Message::Text(msg)).await.is_err() { break; }
                    }
                    Some(TtsCmd::Flush) => {
                        let msg = build("", false);
                        if ws.send(Message::Text(msg)).await.is_err() { break; }
                    }
                    None => break,
                },
                msg = ws.next() => match msg {
                    Some(Ok(Message::Text(t))) => {
                        tracing::debug!("tts raw msg: {t}");
                        let v: serde_json::Value = match serde_json::from_str(&t) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        match v["type"].as_str() {
                            Some("chunk") => {
                                if let Some(b64) = v["data"].as_str() {
                                    if let Ok(bytes) = BASE64.decode(b64) {
                                        let f = decode_pcm(&bytes);
                                        let _ = out_tx.send(TtsOut::Pcm(f)).await;
                                    }
                                }
                            }
                            Some("done") => {
                                let _ = out_tx.send(TtsOut::Done).await;
                                break;
                            }
                            Some("error") => {
                                let _ = out_tx.send(TtsOut::Error(v.to_string())).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(Ok(Message::Binary(b))) => {
                        let f = decode_pcm(&b);
                        let _ = out_tx.send(TtsOut::Pcm(f)).await;
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                },
            }
        }
        let _ = out_tx.send(TtsOut::Done).await;
    });

    Ok(TtsSession {
        cmd,
        out,
        handle: task.abort_handle(),
    })
}