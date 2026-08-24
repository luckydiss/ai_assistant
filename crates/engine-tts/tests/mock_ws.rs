// Mock Cartesia WebSocket server for tests. Runs on 127.0.0.1:<port>.
// Responds to "stream": true with a small base64 pcm chunk, to "stream": false
// with a "done" message.
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

pub struct MockWs {
    pub port: u16,
    pub flush_seen: Arc<AtomicBool>,
    pub context_ids: Arc<Mutex<Vec<String>>>,
}

pub async fn spawn() -> anyhow::Result<MockWs> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let flush_seen = Arc::new(AtomicBool::new(false));
    let context_ids = Arc::new(Mutex::new(Vec::new()));
    let flag = flush_seen.clone();
    let ctx = context_ids.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let flag = flag.clone();
            let ctx = ctx.clone();
            tokio::spawn(async move {
                let mut ws = match tokio_tungstenite::accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(_) => return,
                };
                while let Some(Ok(msg)) = ws.next().await {
                    if let Message::Text(t) = msg {
                        let v: Value = match serde_json::from_str(&t) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        if let Some(id) = v["context_id"].as_str() {
                            ctx.lock().unwrap().push(id.to_string());
                        }
                        if v["continue"] == Value::Bool(true) {
                            let audio = BASE64.encode(
                                [0.0f32, 0.1, 0.2, -0.1, 0.5, -0.5]
                                    .map(f32::to_le_bytes)
                                    .concat(),
                            );
                            let resp = serde_json::json!({ "type": "chunk", "data": audio });
                            if ws.send(Message::Text(resp.to_string())).await.is_err() {
                                break;
                            }
                        } else if v["continue"] == Value::Bool(false) {
                            flag.store(true, Ordering::SeqCst);
                            let resp = serde_json::json!({ "type": "done" });
                            let _ = ws.send(Message::Text(resp.to_string())).await;
                            break;
                        }
                    }
                }
            });
        }
    });
    Ok(MockWs {
        port,
        flush_seen,
        context_ids,
    })
}