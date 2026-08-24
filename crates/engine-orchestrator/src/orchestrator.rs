use engine_context::ContextBuilder;
use engine_dialogue::{Speaker, Turn};
use engine_llm::{AnswerEvent, LlmClient};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::task::AbortHandle;

#[derive(Debug, Clone)]
pub enum OrchEvent {
    Token { gen: u64, text: String },
    Done { gen: u64, text: String },
    Error { gen: u64, message: String },
    Status { gen: u64, state: String },
}

enum Cmd {
    Turn(Turn),
    Partial(String),
    Manual(Option<String>, Option<String>),
    SetActive(String),
    ResetActive,
    SetAuto(bool),
    SetSearch(bool),
    LoadChat(String, Vec<Turn>),
    SetPersist(Option<PersistFn>),
}

pub type PersistFn = Arc<dyn Fn(String, Vec<Turn>) + Send + Sync>;

pub struct Orchestrator {
    cmd_tx: mpsc::UnboundedSender<Cmd>,
    events: broadcast::Sender<OrchEvent>,
}

struct ChatState {
    turns: Vec<Turn>,
    summary: String,
}

impl ChatState {
    fn new() -> Self {
        Self {
            turns: Vec::new(),
            summary: String::new(),
        }
    }
}

struct Inner {
    ctx: ContextBuilder,
    llm: LlmClient,
    chats: HashMap<String, ChatState>,
    active: String,
    auto: bool,
    search_json: String,
    answer: Option<AbortHandle>,
    events: broadcast::Sender<OrchEvent>,
    gen: AtomicU64,
    persist: Option<PersistFn>,
    last_partial: String,
}

impl Orchestrator {
    pub fn new(ctx: ContextBuilder, llm: LlmClient, auto: bool) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (events, _) = broadcast::channel(256);

        let search_json = llm.search_tool_json().to_string();
        let mut chats = HashMap::new();
        chats.insert("default".to_string(), ChatState::new());
        let inner = Inner {
            ctx,
            llm,
            chats,
            active: "default".to_string(),
            auto,
            search_json,
            answer: None,
            events: events.clone(),
            gen: AtomicU64::new(0),
            persist: None,
            last_partial: String::new(),
        };

        tokio::spawn(async move {
            let inner = Arc::new(Mutex::new(inner));
            let mut rx = cmd_rx;
            while let Some(cmd) = rx.recv().await {
                let mut g = inner.lock().await;
                g.handle(cmd, &inner).await;
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

    pub fn on_partial(&self, text: String) {
        let _ = self.cmd_tx.send(Cmd::Partial(text));
    }

    pub fn manual(&self, note: Option<String>, image_b64: Option<String>) {
        let _ = self.cmd_tx.send(Cmd::Manual(note, image_b64));
    }

    pub fn set_active_chat(&self, id: String) {
        let _ = self.cmd_tx.send(Cmd::SetActive(id));
    }

    pub fn reset_active(&self) {
        let _ = self.cmd_tx.send(Cmd::ResetActive);
    }

    pub fn set_auto(&self, on: bool) {
        let _ = self.cmd_tx.send(Cmd::SetAuto(on));
    }

    pub fn set_search(&self, on: bool) {
        let _ = self.cmd_tx.send(Cmd::SetSearch(on));
    }

    /// Persist callback: invoked whenever a chat's turns change (chat_id, turns).
    pub fn set_persist(&self, f: Option<PersistFn>) {
        let _ = self.cmd_tx.send(Cmd::SetPersist(f));
    }

    /// Restore a chat's history from storage.
    pub fn load_chat(&self, chat_id: String, turns: Vec<Turn>) {
        let _ = self.cmd_tx.send(Cmd::LoadChat(chat_id, turns));
    }
}

impl Inner {
    fn active_state(&mut self) -> &mut ChatState {
        self.chats.entry(self.active.clone()).or_insert_with(ChatState::new)
    }

    async fn handle(&mut self, cmd: Cmd, inner: &Arc<Mutex<Inner>>) {
        match cmd {
            Cmd::Turn(turn) => {
                let auto = self.auto;
                let chat_id = self.active.clone();
                let st = self.active_state();
                st.turns.push(turn.clone());
                if st.turns.len() > 12 {
                    st.turns.remove(0);
                }
                self.maybe_persist(&chat_id);
                let gen = self.gen.load(Ordering::SeqCst);
                let _ = self.events.send(OrchEvent::Status {
                    gen,
                    state: "listening".into(),
                });
                if auto && turn.speaker == Speaker::Interviewer {
                    self.fire(None, None, inner).await;
                }
            }
            Cmd::Partial(text) => self.last_partial = text,
            Cmd::Manual(note, image) => self.fire(note, image, inner).await,
            Cmd::SetActive(id) => {
                self.chats.entry(id.clone()).or_insert_with(ChatState::new);
                self.active = id;
            }
            Cmd::ResetActive => {
                let chat_id = self.active.clone();
                if let Some(st) = self.chats.get_mut(&self.active) {
                    st.turns.clear();
                    st.summary.clear();
                }
                self.maybe_persist(&chat_id);
            }
            Cmd::SetAuto(on) => {
                self.auto = on;
            }
            Cmd::SetSearch(on) => {
                self.llm.set_search(on, self.search_json.clone());
            }
            Cmd::LoadChat(chat_id, turns) => {
                let st = self.chats.entry(chat_id.clone()).or_insert_with(ChatState::new);
                st.turns = turns;
                st.summary.clear();
            }
            Cmd::SetPersist(f) => {
                self.persist = f;
            }
        }
    }

    fn maybe_persist(&self, chat_id: &str) {
        if let Some(f) = &self.persist {
            let turns = self
                .chats
                .get(chat_id)
                .map(|c| c.turns.clone())
                .unwrap_or_default();
            f(chat_id.to_string(), turns);
        }
    }

    async fn fire(
        &mut self,
        note: Option<String>,
        image_b64: Option<String>,
        inner: &Arc<Mutex<Inner>>,
    ) {
        if let Some(a) = self.answer.take() {
            a.abort();
        }

        let gen = self.gen.fetch_add(1, Ordering::SeqCst) + 1;
        let chat_id = self.active.clone();
        let (turns, summary) = {
            let st = self.active_state();
            (st.turns.clone(), st.summary.clone())
        };
        let last_i = turns
            .iter()
            .rev()
            .find(|t| t.speaker == Speaker::Interviewer)
            .cloned();
        let partial = self.last_partial.trim();
        let live_partial = !partial.is_empty()
            && last_i.as_ref().map(|t| t.text.as_str()) != Some(partial);
        let focus = if live_partial {
            let now = chrono::Utc::now();
            Some(Turn {
                speaker: Speaker::Interviewer,
                text: self.last_partial.clone(),
                start_time: now,
                end_time: now,
            })
        } else {
            last_i
        };
        let focus_was_partial = live_partial;

        if note.is_none() && image_b64.is_none() && turns.is_empty() && !live_partial {
            let _ = self.events.send(OrchEvent::Error {
                gen,
                message: "Нет контекста: дождитесь вопроса интервьюера.".into(),
            });
            return;
        }

        if let Some(n) = note.as_deref() {
            if !n.trim().is_empty() {
                let now = chrono::Utc::now();
                let st = self.chats.entry(chat_id.clone()).or_insert_with(ChatState::new);
                st.turns.push(Turn {
                    speaker: Speaker::Interviewer,
                    text: n.to_string(),
                    start_time: now,
                    end_time: now,
                });
                if st.turns.len() > 12 {
                    st.turns.remove(0);
                }
                self.maybe_persist(&chat_id);
            }
        }

        let messages = self.ctx.build(
            &summary,
            &turns,
            focus.as_ref(),
            focus_was_partial,
            note.as_deref(),
            image_b64.as_deref(),
        );
        let (mut rx, handle) = self.llm.stream_answer(messages);
        self.answer = Some(handle);
        let events = self.events.clone();
        let inner2 = inner.clone();
        let chat_id2 = chat_id.clone();

        tokio::spawn(async move {
            while let Some(ev) = rx.recv().await {
                match ev {
                    AnswerEvent::Token(t) => {
                        let _ = events.send(OrchEvent::Token { gen, text: t });
                    }
                    AnswerEvent::Done(text) => {
                        if text.trim().is_empty() {
                            let _ = events.send(OrchEvent::Error {
                                gen,
                                message: "Модель вернула пустой ответ.".into(),
                            });
                        } else {
                            let now = chrono::Utc::now();
                            let mut g = inner2.lock().await;
                            if let Some(st) = g.chats.get_mut(&chat_id2) {
                                st.turns.push(Turn {
                                    speaker: Speaker::Candidate,
                                    text: text.clone(),
                                    start_time: now,
                                    end_time: now,
                                });
                                if st.turns.len() > 12 {
                                    st.turns.remove(0);
                                }
                                g.maybe_persist(&chat_id2);
                            }
                            drop(g);
                            let _ = events.send(OrchEvent::Done { gen, text });
                        }
                    }
                    AnswerEvent::Failed(e) => {
                        let _ = events.send(OrchEvent::Error { gen, message: e });
                    }
                }
            }
        });
        let _ = self.events.send(OrchEvent::Status {
            gen,
            state: "generating".into(),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerKind {
    Auto,
    Speculative,
    Manual,
}

/// Чистое решение о триггере. Используется в replay-симуляции.
/// Live-цикл работает в manual-only режиме, поэтому здесь функция НЕ вызывается.
pub fn trigger_decision(
    turn: &Turn,
    min_words: usize,
    debounce_ms: u64,
) -> Option<(TriggerKind, u64)> {
    if turn.speaker != Speaker::Interviewer {
        return None;
    }
    let words = turn.text.split_whitespace().count();
    if words < min_words {
        return None;
    }
    if words > 14 && turn.text.contains('?') {
        return Some((TriggerKind::Speculative, 200));
    }
    Some((TriggerKind::Auto, debounce_ms))
}

/// Разрешён ли чанк данного lane в обработку.
/// mode="manual": только при активной записи (record);
/// mode="vad": любой источник (решающий — VAD).
pub fn gate(mode: &str, recording: bool, source: &str, is_mic: bool) -> bool {
    let lane_ok = match source {
        "system" => !is_mic,
        "mic" => is_mic,
        _ => true,
    };
    if !lane_ok {
        return false;
    }
    match mode {
        "manual" => recording,
        _ => true,
    }
}
