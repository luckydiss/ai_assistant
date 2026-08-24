use crate::{extract_delta, parse_sse_line};
use engine_context::ChatMessage;
use futures::StreamExt;
use reqwest::Client;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::AbortHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerEvent {
    Token(String),
    Done(String),
    Failed(String),
}

pub struct LlmClient {
    http: Client,
    base_url: String,
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
    reasoning_effort: Option<String>,
    search_enabled: bool,
    search_tool_json: String,
}

impl LlmClient {
    pub fn new(
        base_url: String,
        api_key: String,
        model: String,
        temperature: f32,
        max_tokens: u32,
        reasoning_effort: Option<String>,
    ) -> Result<Self, reqwest::Error> {
        Ok(Self {
            http: Client::builder()
                .http1_only()
                .timeout(Duration::from_secs(60))
                .user_agent("curl/8.0")
                .build()?,
            base_url,
            api_key,
            model,
            temperature,
            max_tokens,
            reasoning_effort,
            search_enabled: false,
            search_tool_json: String::new(),
        })
    }

    pub fn with_search(mut self, enabled: bool, tool_json: String) -> Self {
        self.search_enabled = enabled;
        self.search_tool_json = tool_json;
        self
    }

    pub fn set_search(&mut self, enabled: bool, tool_json: String) {
        self.search_enabled = enabled;
        self.search_tool_json = tool_json;
    }

    pub fn search_tool_json(&self) -> &str {
        &self.search_tool_json
    }

    pub fn stream_answer(
        &self,
        messages: Vec<ChatMessage>,
    ) -> (mpsc::Receiver<AnswerEvent>, AbortHandle) {
        let (tx, rx) = mpsc::channel(64);
        let http = self.http.clone();
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let temperature = self.temperature;
        let max_tokens = self.max_tokens;
        let reasoning_effort = self.reasoning_effort.clone();
        let search_enabled = self.search_enabled;
        let search_tool_json = self.search_tool_json.clone();

        let task = tokio::spawn(async move {
            run(
                http,
                base_url,
                api_key,
                model,
                temperature,
                max_tokens,
                reasoning_effort,
                search_enabled,
                search_tool_json,
                messages,
                tx,
            )
            .await;
        });

        (rx, task.abort_handle())
    }
}

#[allow(clippy::too_many_arguments)]
async fn run(
    http: Client,
    base_url: String,
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
    reasoning_effort: Option<String>,
    search_enabled: bool,
    search_tool_json: String,
    messages: Vec<ChatMessage>,
    tx: mpsc::Sender<AnswerEvent>,
) {
    let mut body = serde_json::json!({
        "model": model, "messages": messages, "temperature": temperature,
        "max_tokens": max_tokens, "stream": true
    });
    if let Some(e) = reasoning_effort {
        body["reasoning_effort"] = serde_json::json!(e);
    }
    if search_enabled && !search_tool_json.is_empty() {
        if let Ok(extra) = serde_json::from_str::<serde_json::Value>(&search_tool_json) {
            if let (Some(b), Some(e)) = (body.as_object_mut(), extra.as_object()) {
                for (k, v) in e {
                    b.insert(k.clone(), v.clone());
                }
            }
        }
    }

    let resp = match http
        .post(format!("{}/chat/completions", base_url))
        .bearer_auth(&api_key)
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(AnswerEvent::Failed(e.to_string())).await;
            return;
        }
    };

    if resp.status().as_u16() == 401 {
        let _ = tx.send(AnswerEvent::Failed("auth".into())).await;
        return;
    }
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let _ = tx
            .send(AnswerEvent::Failed(format!("http {status}: {}", body.trim())))
            .await;
        return;
    }

    let mut stream = resp.bytes_stream();
    let mut line_buf = String::new();
    let mut full = String::new();

    while let Some(item) = stream.next().await {
        let chunk = match item {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(AnswerEvent::Failed(e.to_string())).await;
                return;
            }
        };
        line_buf.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = line_buf.find('\n') {
            let line: String = line_buf.drain(..=pos).collect();
            let Some(payload) = parse_sse_line(&line) else {
                continue;
            };
            if payload == "[DONE]" {
                let _ = tx.send(AnswerEvent::Done(full.clone())).await;
                return;
            }
            let Some(delta) = extract_delta(payload) else {
                continue;
            };
            full.push_str(&delta);
            let _ = tx.send(AnswerEvent::Token(delta)).await;
        }
    }
    let _ = tx.send(AnswerEvent::Done(full)).await;
}
