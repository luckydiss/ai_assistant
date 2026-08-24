# Design: LLM Client

## 1. Cargo.toml

```toml
[package]
name = "engine-llm"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
serde_json.workspace = true
reqwest.workspace = true
tokio.workspace = true
futures.workspace = true
engine-context = { path = "../engine-context" }
```

## 2. src/lib.rs

```rust
//! OpenAI-compatible streaming LLM client with SKIP protocol
#![deny(clippy::all)]

mod client;
mod skip;
mod sse;

pub use client::*;
pub use skip::*;
pub use sse::*;
```

## 3. src/sse.rs

```rust
/// Возвращает payload SSE-строки ("data: X" -> "X"), иначе None.
pub fn parse_sse_line(line: &str) -> Option<&str> {
    line.trim().strip_prefix("data:").map(|p| p.trim())
}

/// Извлекает choices[0].delta.content из JSON-дельты.
pub fn extract_delta(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v["choices"][0]["delta"]["content"]
        .as_str()
        .map(|s| s.to_string())
}
```

## 4. src/skip.rs

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipState {
    Buffering,
    Skipped,
    Passthrough,
}

const SKIP: &str = "<SKIP>";

/// Кормит буфер дельтой и возвращает состояние протокола.
pub fn feed_skip(buf: &mut String, delta: &str) -> SkipState {
    buf.push_str(delta);
    let t = buf.trim_start();
    if t.len() < SKIP.len() {
        return if SKIP.starts_with(t) { SkipState::Buffering } else { SkipState::Passthrough };
    }
    if t.starts_with(SKIP) { SkipState::Skipped } else { SkipState::Passthrough }
}
```

## 5. src/client.rs

```rust
use crate::{feed_skip, extract_delta, parse_sse_line, SkipState};
use engine_context::ChatMessage;
use futures::StreamExt;
use reqwest::Client;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::AbortHandle;

#[derive(Debug, Clone)]
pub enum AnswerEvent {
    Token(String),
    Done(String),
    Skipped,
    Failed(String),
}

pub struct LlmClient {
    http: Client,
    base_url: String,
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

impl LlmClient {
    pub fn new(base_url: String, api_key: String, model: String, temperature: f32, max_tokens: u32) -> Self {
        Self {
            http: Client::builder().timeout(Duration::from_secs(60)).build().unwrap(),
            base_url, api_key, model, temperature, max_tokens,
        }
    }

    pub fn stream_answer(&self, messages: Vec<ChatMessage>) -> (mpsc::Receiver<AnswerEvent>, AbortHandle) {
        let (tx, rx) = mpsc::channel(64);
        let http = self.http.clone();
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let temperature = self.temperature;
        let max_tokens = self.max_tokens;

        let task = tokio::spawn(async move {
            run(http, base_url, api_key, model, temperature, max_tokens, messages, tx).await;
        });

        (rx, task.abort_handle())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Phase { SkipCheck, Streaming }

async fn run(
    http: Client, base_url: String, api_key: String, model: String,
    temperature: f32, max_tokens: u32, messages: Vec<ChatMessage>, tx: mpsc::Sender<AnswerEvent>,
) {
    let body = serde_json::json!({
        "model": model, "messages": messages, "temperature": temperature,
        "max_tokens": max_tokens, "stream": true
    });

    let resp = match http
        .post(format!("{}/chat/completions", base_url))
        .bearer_auth(&api_key)
        .json(&body)
        .send().await
    {
        Ok(r) => r,
        Err(e) => { let _ = tx.send(AnswerEvent::Failed(e.to_string())).await; return; }
    };

    if resp.status().as_u16() == 401 {
        let _ = tx.send(AnswerEvent::Failed("auth".into())).await; return;
    }
    if !resp.status().is_success() {
        let _ = tx.send(AnswerEvent::Failed(format!("http {}", resp.status()))).await; return;
    }

    let mut stream = resp.bytes_stream();
    let mut line_buf = String::new();
    let mut skip_buf = String::new();
    let mut phase = Phase::SkipCheck;
    let mut full = String::new();

    while let Some(item) = stream.next().await {
        let chunk = match item {
            Ok(c) => c,
            Err(e) => { let _ = tx.send(AnswerEvent::Failed(e.to_string())).await; return; }
        };
        line_buf.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = line_buf.find('\n') {
            let line: String = line_buf.drain(..=pos).collect();
            let Some(payload) = parse_sse_line(&line) else { continue };
            if payload == "[DONE]" {
                let _ = tx.send(AnswerEvent::Done(full.clone())).await; return;
            }
            let Some(delta) = extract_delta(payload) else { continue };
            full.push_str(&delta);

            match phase {
                Phase::SkipCheck => match feed_skip(&mut skip_buf, &delta) {
                    SkipState::Buffering => {}
                    SkipState::Skipped => { let _ = tx.send(AnswerEvent::Skipped).await; return; }
                    SkipState::Passthrough => {
                        phase = Phase::Streaming;
                        let _ = tx.send(AnswerEvent::Token(std::mem::take(&mut skip_buf))).await;
                    }
                },
                Phase::Streaming => { let _ = tx.send(AnswerEvent::Token(delta)).await; }
            }
        }
    }
    let _ = tx.send(AnswerEvent::Done(full)).await;
}
```

## 6. Mock SSE server (tests helper)

`crates/engine-llm/tests/mock.rs` (подключить как `mod mock;` в каждом тест-файле):

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub fn sse_body(deltas: &[&str]) -> String {
    let mut s = String::new();
    for d in deltas {
        s.push_str(&format!("data: {{\"choices\":[{{\"delta\":{{\"content\":\"{}\"}}}}]}}\n\n", d));
    }
    s.push_str("data: [DONE]\n\n");
    s
}

pub async fn spawn_mock_sse(body: String, delay_ms: u64) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let c2 = count.clone();

    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else { return };
            let c2 = c2.clone();
            let body = body.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 8192];
                let _ = sock.read(&mut buf).await;
                c2.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                );
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(body.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });

    (format!("http://{}", addr), count)
}
```

## Рассмотрено и отклонено
- **async-openai crate:** отклонено — лишняя зависимость, SSE парсится вручную за 40 строк
- **Retries в стриме:** отклонено — оркестратор сам перезапрашивает новым триггером
