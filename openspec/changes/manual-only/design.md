# Design: Manual-Only

Правки уже написанного кода (changes 002/007/008/009/011). Даём ПОЛНЫЕ замены файлов, где код уменьшается.

## 1. engine-config/src/config.rs

Удалить OrchestratorConfig и поле orchestrator из Config. В LlmConfig добавить:

```rust
#[serde(default)] pub reasoning_effort: Option<String>,
```

validate(): убрать проверку min_words; оставить silence_ms>0 и temperature 0..=2.
Пример config.toml обновить (см. §6).

## 2. engine-context/src/builder.rs

Новая сигнатура и содержимое:

```rust
pub fn build(&self, summary: &str, turns: &[Turn], focus: Option<&Turn>, note: Option<&str>) -> Vec<ChatMessage> {
    let system = format!("{}\n\nPersona кандидата: {}", self.system, self.persona);
    let mut kept = turns.to_vec();
    loop {
        let user = self.user_content(summary, &kept, focus, note);
        let total = crate::estimate_tokens(&system) + crate::estimate_tokens(&user);
        if total <= self.max_tokens || kept.is_empty() {
            return vec![
                ChatMessage { role: Role::System, content: system },
                ChatMessage { role: Role::User, content: user },
            ];
        }
        kept.remove(0);
    }
}

fn user_content(&self, summary: &str, turns: &[Turn], focus: Option<&Turn>, note: Option<&str>) -> String {
    let mut s = String::new();
    if !summary.is_empty() { s.push_str(&format!("Раньше: {}\n", summary)); }
    s.push_str("Диалог:\n");
    for t in turns {
        let tag = match t.speaker { Speaker::Interviewer => "I", Speaker::Candidate => "C" };
        s.push_str(&format!("{}: {}\n", tag, t.text));
    }
    if let Some(f) = focus { s.push_str(&format!("Последний вопрос I: «{}»\n", f.text)); }
    if let Some(n) = note { s.push_str(&format!("Комментарий пользователя: {}\n", n)); }
    s.push_str("Ответь по запросу кандидата.");
    s
}
```

Убрать любые строки «Ответ по протоколу.» и упоминания SKIP.

## 3. engine-llm

- Удалить `src/skip.rs` и `pub use skip::*` из lib.rs.
- `AnswerEvent`: удалить вариант `Skipped`.
- `client.rs`: удалить `Phase`, `skip_buf`, вызовы feed_skip; в цикле SSE после extract_delta сразу:

```rust
let Some(delta) = extract_delta(payload) else { continue };
full.push_str(&delta);
let _ = tx.send(AnswerEvent::Token(delta)).await;
```

- В body запроса добавить:

```rust
if let Some(re) = &self.reasoning_effort { body["reasoning_effort"] = serde_json::json!(re); }
```

(LlmClient::new принимает reasoning_effort: Option<String>.)

## 4. engine-orchestrator/src/orchestrator.rs — ПОЛНАЯ ЗАМЕНА

```rust
use engine_context::ContextBuilder;
use engine_dialogue::{Speaker, Turn};
use engine_llm::{AnswerEvent, LlmClient};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::task::AbortHandle;

#[derive(Debug, Clone)]
pub enum OrchEvent { Token(String), Done, Error(String), Status(String) }

enum Cmd { Turn(Turn), Manual(Option<String>) }

pub struct Orchestrator {
    cmd_tx: mpsc::UnboundedSender<Cmd>,
    events: broadcast::Sender<OrchEvent>,
}

struct Inner {
    ctx: ContextBuilder,
    llm: LlmClient,
    turns: Vec<Turn>,
    summary: String,
    answer: Option<AbortHandle>,
    events: broadcast::Sender<OrchEvent>,
}

impl Orchestrator {
    pub fn new(ctx: ContextBuilder, llm: LlmClient) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (events, _) = broadcast::channel(256);
        let inner = Inner { ctx, llm, turns: Vec::new(), summary: String::new(),
                            answer: None, events: events.clone() };
        tokio::spawn(async move {
            let inner = Arc::new(Mutex::new(inner));
            while let Some(cmd) = cmd_rx.recv().await {
                inner.lock().await.handle(cmd).await;
            }
        });
        Self { cmd_tx, events }
    }
    pub fn subscribe(&self) -> broadcast::Receiver<OrchEvent> { self.events.subscribe() }
    pub fn on_turn(&self, turn: Turn) { let _ = self.cmd_tx.send(Cmd::Turn(turn)); }
    pub fn manual(&self, note: Option<String>) { let _ = self.cmd_tx.send(Cmd::Manual(note)); }
}

impl Inner {
    async fn handle(&mut self, cmd: Cmd) {
        match cmd {
            Cmd::Turn(turn) => {
                self.turns.push(turn);
                if self.turns.len() > 12 { self.turns.remove(0); }
                let _ = self.events.send(OrchEvent::Status("listening".into()));
            }
            Cmd::Manual(note) => self.fire(note).await,
        }
    }

    async fn fire(&mut self, note: Option<String>) {
        if let Some(a) = self.answer.take() { a.abort(); } // last-trigger-wins
        let focus = self.turns.iter().rev()
            .find(|t| t.speaker == Speaker::Interviewer).cloned();
        let messages = self.ctx.build(&self.summary, &self.turns, focus.as_ref(), note.as_deref());
        let (mut rx, handle) = self.llm.stream_answer(messages);
        self.answer = Some(handle);
        let events = self.events.clone();
        tokio::spawn(async move {
            while let Some(ev) = rx.recv().await {
                match ev {
                    AnswerEvent::Token(t)  => { let _ = events.send(OrchEvent::Token(t)); }
                    AnswerEvent::Done(_)   => { let _ = events.send(OrchEvent::Done); }
                    AnswerEvent::Failed(e) => { let _ = events.send(OrchEvent::Error(e)); }
                }
            }
        });
        let _ = self.events.send(OrchEvent::Status("generating".into()));
    }
}
```

Удалить из crate: trigger-таймеры, debounce, min_words, speculative, всё про SKIP.

## 5. Тесты: что удалить / добавить

- engine-orchestrator: УДАЛИТЬ triggers_after_debounce, short_turn_ignored, speculative_trigger_fast, skip_hidden_from_ui. ПЕРЕИМЕНОВАТЬ new_trigger_cancels_previous → manual_cancels_previous (два manual подряд). ДОБАВИТЬ turns_accumulate_no_fire, manual_includes_context (mock с захватом тела).
- engine-llm: УДАЛИТЬ skip_detected, passthrough_after_partial, skip_emits_no_tokens. ДОБАВИТЬ reasoning_effort_sent.
- engine-context: УДАЛИТЬ includes_skip_protocol; обновить вызовы build(..., Some(&focus), ...) / (..., None, ...); ДОБАВИТЬ builds_without_focus, no_skip_instruction.
- engine-config: applies_defaults/validates_thresholds поправить под удалённую секцию.

## 6. config.toml (заменить)

```toml
[stt]
provider = "groq"
api_key = "gsk_..."
model = "whisper-large-v3-turbo"
chunk_ms = 7000

[llm]
provider = "openai"
api_key = "sk-..."
model = "deepseek-v4-flash-0731"
base_url = "https://api.dslab.tech/v1"
temperature = 0
reasoning_effort = "low"
max_tokens = 700

[vad]
silence_ms = 600
max_segment_ms = 7000

[prompts]
system = """Ты — невидимый ассистент на техническом собеседовании.
Даётся диалог: I — интервьюер, C — кандидат.
Кандидат сам запрашивает подсказку, когда нужна помощь.
Отвечай сразу суть: 2–5 буллетов или короткий код-блок.
Язык = язык последнего вопроса I или языка запроса.
Без вступлений и мета-комментариев."""
persona = "Senior Rust developer, 5 years experience"
```

## 7. Wiring (main.rs) и overlay.js

- main.rs: `Orchestrator::new(ctx, llm)` (без debounce/min_words); LlmClient::new(..., cfg.llm.reasoning_effort.clone()); убрать OrchEvent::Skipped из форвардера.
- overlay.js: удалить listener answer_skipped; в статусе listening рендерить подсказку «Что сказать — Ctrl+Shift+Space».

## Рассмотрено и отклонено
- **Флаг trigger_mode:** отклонено — на этой стадии удаление дешевле; возврат = новый change
- **Оставить skip.rs «на будущее»:** отклонено — мёртвый код галлюцинируется агентом как живой
