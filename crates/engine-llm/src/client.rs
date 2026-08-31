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

#[derive(Clone)]
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
    /// Каталог моделей провайдера (для валидации при смене модели).
    catalog: Option<std::sync::Arc<dyn engine_models::ModelProvider>>,
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
            catalog: None,
        })
    }

    /// Подключить каталог провайдера (валидация модели при set_model).
    pub fn with_catalog(
        mut self,
        catalog: std::sync::Arc<dyn engine_models::ModelProvider>,
    ) -> Self {
        self.catalog = Some(catalog);
        self
    }

    /// Err, если модель отсутствует в каталоге провайдера.
    /// Каталог не подключён или недоступен → Ok.
    pub async fn validate_model(&self) -> Result<(), String> {
        match &self.catalog {
            None => Ok(()),
            Some(c) => c
                .validate_model(&self.model)
                .await
                .map_err(|e| e.to_string()),
        }
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

    /// Смена модели/уровня рассуждений на живом клиенте.
    pub fn set_model(&mut self, model: String, reasoning_effort: Option<String>) {
        self.model = model;
        self.reasoning_effort = reasoning_effort;
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

    /// Non-streaming completion (используется для фоновой суммаризации).
    pub async fn complete(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String, String> {
        self.complete_with(&self.model, messages, max_tokens, temperature)
            .await
    }

    /// Как complete, но с явной моделью (context.summary_model).
    pub async fn complete_with(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String, String> {
        let mut body = serde_json::json!({
            "model": model, "messages": messages,
            "temperature": temperature
        });
        if max_tokens > 0 {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        let url = format!("{}/chat/completions", self.base_url);
        let mut resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().as_u16() == 400 {
            let txt = resp.text().await.unwrap_or_default();
            if txt.contains("compatibility policy") || txt.contains("temperature") {
                if let Some(o) = body.as_object_mut() {
                    o.remove("temperature");
                }
                resp = self
                    .http
                    .post(&url)
                    .bearer_auth(&self.api_key)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
            } else {
                return Err(format!("http 400: {}", txt.trim()));
            }
        }
        if !resp.status().is_success() {
            return Err(format!("http {}", resp.status()));
        }
        let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok(v["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string())
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
            "stream": true
        });
        // max_tokens = 0 → лимит не отправляется (потолок модели).
        if max_tokens > 0 {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
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

    async fn send_req(
        http: &reqwest::Client,
        base_url: &str,
        api_key: &str,
        body: &serde_json::Value,
    ) -> reqwest::Result<reqwest::Response> {
        http.post(format!("{}/chat/completions", base_url))
            .bearer_auth(api_key)
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .json(body)
            .send()
            .await
    }

    // Некоторые модели (например GPT-5.6) отвергают sampling-параметры —
    // при 400 «compatibility policy» повторяем без temperature.
    let (resp, _body) = match send_req(&http, &base_url, &api_key, &body).await {
        Ok(r) if r.status().as_u16() == 400 => {
            let txt = r.text().await.unwrap_or_default();
            if txt.contains("compatibility policy") || txt.contains("temperature") {
                if let Some(o) = body.as_object_mut() {
                    o.remove("temperature");
                }
                match send_req(&http, &base_url, &api_key, &body).await {
                    Ok(r2) => (r2, body),
                    Err(e) => {
                        let _ = tx.send(AnswerEvent::Failed(e.to_string())).await;
                        return;
                    }
                }
            } else {
                let _ = tx
                    .send(AnswerEvent::Failed(format!("http 400: {}", txt.trim())))
                    .await;
                return;
            }
        }
        Ok(r) => (r, body),
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
