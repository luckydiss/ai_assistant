# Design: Orchestrator

## 1. Cargo.toml

```toml
[package]
name = "engine-orchestrator"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
tokio.workspace = true
engine-context = { path = "../engine-context" }
engine-llm = { path = "../engine-llm" }
engine-dialogue = { path = "../engine-dialogue" }
```

## 2. src/lib.rs

```rust
//! Trigger engine: decides when to ask the LLM
#![deny(clippy::all)]

mod orchestrator;
pub use orchestrator::*;
```

## 3. src/orchestrator.rs

```rust
use engine_context::ContextBuilder;
use engine_dialogue::{Speaker, Turn};
use engine_llm::{AnswerEvent, LlmClient};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::task::AbortHandle;

#[derive(Debug, Clone)]
pub enum OrchEvent {
    Token(String),
    Done,
    Skipped,
    Error(String),
    Status(String),
}

enum Cmd {
    Turn(Turn),
    Manual(Option<String>),
    Fire(Option<String>),
}

pub struct Orchestrator {
    cmd_tx: mpsc::UnboundedSender<Cmd>,
    events: broadcast::Sender<OrchEvent>,
}

struct Inner {
    ctx: ContextBuilder,
    llm: LlmClient,
    debounce_ms: u64,
    min_words: usize,
    turns: Vec<Turn>,
    summary: String,
    trigger: Option<AbortHandle>,
    answer: Option<AbortHandle>,
    events: broadcast::Sender<OrchEvent>,
    cmd_tx: mpsc::UnboundedSender<Cmd>,
}

impl Orchestrator {
    pub fn new(
        ctx: ContextBuilder,
        llm: LlmClient,
        debounce_ms: u64,
        min_words: usize,
    ) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (events, _) = broadcast::channel(256);

        let inner = Inner {
            ctx, llm, debounce_ms, min_words,
            turns: Vec::new(),
            summary: String::new(),
            trigger: None, answer: None,
            events: events.clone(),
            cmd_tx: cmd_tx.clone(),
        };

        tokio::spawn(async move {
            let inner = Arc::new(Mutex::new(inner));
            let mut rx = cmd_rx;
            while let Some(cmd) = rx.recv().await {
                let mut g = inner.lock().await;
                g.handle(cmd).await;
            }
        });

        Self { cmd_tx, events }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<OrchEvent> {
        self.events.subscribe()
    }

    pub fn on_turn(&self, turn: Turn) {
        let _ = self.cmd_tx.send(Cmd::Turn(turn));
    }

    pub fn manual(&self, note: Option<String>) {
        let _ = self.cmd_tx.send(Cmd::Manual(note));
    }

    pub fn set_summary(&self, s: String) {
        // summary обновляется через Fire-цикл; публичный сеттер не нужен —
        // оставлен как заглушка НЕ РЕАЛИЗОВЫВАТЬ, если спека не требует
        let _ = s;
    }
}

impl Inner {
    async fn handle(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Turn(turn) => self.on_turn(turn),
            Cmd::Manual(note) => self.fire(note).await,
            Cmd::Fire(note) => self.fire(note).await,
        }
    }

    fn on_turn(&mut self, turn: Turn) {
        self.turns.push(turn.clone());
        if self.turns.len() > 12 { self.turns.remove(0); }

        if turn.speaker != Speaker::Interviewer { return; }

        let words = turn.text.split_whitespace().count();
        if words < self.min_words { return; }

        let speculative = words > 14 && turn.text.contains('?');
        let delay = if speculative { 200 } else { self.debounce_ms };

        if let Some(t) = self.trigger.take() { t.abort(); }
        let tx = self.cmd_tx.clone();
        let h = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay)).await;
            let _ = tx.send(Cmd::Fire(None));
        }).abort_handle();
        self.trigger = Some(h);
        let _ = self.events.send(OrchEvent::Status("listening".into()));
    }

    async fn fire(&mut self, note: Option<String>) {
        self.trigger.take().map(|t| t.abort());

        let Some(focus) = self.turns.iter().rev().find(|t| t.speaker == Speaker::Interviewer).cloned()
        else { return; };

        if let Some(a) = self.answer.take() { a.abort(); } // last-trigger-wins

        let messages = self.ctx.build(&self.summary, &self.turns, &focus, note.as_deref());
        let (mut rx, handle) = self.llm.stream_answer(messages);
        self.answer = Some(handle);
        let events = self.events.clone();

        tokio::spawn(async move {
            while let Some(ev) = rx.recv().await {
                match ev {
                    AnswerEvent::Token(t) => { let _ = events.send(OrchEvent::Token(t)); }
                    AnswerEvent::Done(_) => { let _ = events.send(OrchEvent::Done); }
                    AnswerEvent::Skipped => { let _ = events.send(OrchEvent::Skipped); }
                    AnswerEvent::Failed(e) => { let _ = events.send(OrchEvent::Error(e)); }
                }
            }
        });
        let _ = self.events.send(OrchEvent::Status("generating".into()));
    }
}
```

## 4. Заметки по тестам

Тесты используют mock из `engine-llm/tests/mock.rs` — скопировать файл в `crates/engine-orchestrator/tests/mock.rs` без изменений.

Пример теста `triggers_after_debounce`:

```rust
mod mock;

use engine_context::ContextBuilder;
use engine_dialogue::{Speaker, Turn};
use engine_llm::LlmClient;
use engine_orchestrator::{OrchEvent, Orchestrator};
use chrono::Utc;

fn turn(speaker: Speaker, text: &str) -> Turn {
    Turn { speaker, text: text.into(), start_time: Utc::now(), end_time: Utc::now() }
}

#[tokio::test]
async fn triggers_after_debounce() {
    let (url, count) = mock::spawn_mock_sse(mock::sse_body(&["Пр", "ивет"]), 0).await;
    let llm = LlmClient::new(url, "k".into(), "m".into(), 0.4, 100);
    let ctx = ContextBuilder::new("SYS <SKIP>".into(), "persona".into(), 4000);
    let orch = Orchestrator::new(ctx, llm, 100, 4);

    let mut rx = orch.subscribe();
    orch.on_turn(turn(Speaker::Interviewer, "как работает event loop в node js и почему это важно"));

    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
}
```

## Рассмотрено и отклонено
- **Трейт AnswerPort для тестов:** отклонено — mock SSE-сервер тестирует реальный HTTP-путь
- **Summary-сеттер:** отклонено — summary передаётся через wiring в change 010
