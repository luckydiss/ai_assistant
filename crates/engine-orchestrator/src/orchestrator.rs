use engine_context::{ChatMessage, ContextBuilder, ContextInput, MessageContent, Role};
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
    Cancel { keep: bool },
    SetCancelPolicy { on_resend: bool, keep: bool },
    SummaryDone(String),
    SetCtx(ContextBuilder),
    SetLlm {
        model: String,
        effort: Option<String>,
    },
    SetMemory {
        recent_window: usize,
        key_turns_cap: usize,
        summary_max_tokens: u32,
        summary_model: String,
    },
}

/// Реплика — «ключевая», если содержит вопрос/техн. маркер или длинная.
pub fn is_key_turn(t: &Turn) -> bool {
    const MARKERS: &[&str] = &[
        "напиши",
        "объясни",
        "расскажи",
        "как работает",
        "почему",
        "что будет",
        "код",
        "сложность",
        "нарисуй",
        "спроектируй",
    ];
    let text = t.text.to_lowercase();
    MARKERS.iter().any(|m| text.contains(m)) || t.text.chars().count() > 200
}

pub type PersistFn = Arc<dyn Fn(String, Vec<Turn>) + Send + Sync>;

pub struct Orchestrator {
    cmd_tx: mpsc::UnboundedSender<Cmd>,
    events: broadcast::Sender<OrchEvent>,
}

struct ChatState {
    turns: Vec<Turn>,
    key_turns: Vec<Turn>,
    summary: String,
}

impl ChatState {
    fn new() -> Self {
        Self {
            turns: Vec::new(),
            key_turns: Vec::new(),
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
    cur_gen: u64,
    gen_buf: Option<Arc<Mutex<String>>>,
    cancel_on_resend: bool,
    cancel_keep: bool,
    recent_window: usize,
    key_cap: usize,
    summary_max_tokens: u32,
    summary_model: String,
    cmd_tx: mpsc::UnboundedSender<Cmd>,
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
            cur_gen: 0,
            gen_buf: None,
            cancel_on_resend: true,
            cancel_keep: false,
            recent_window: 12,
            key_cap: 12,
            summary_max_tokens: 300,
            summary_model: String::new(),
            cmd_tx: cmd_tx.clone(),
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

    /// Настройки трёхслойной памяти (028). Вызывать после new().
    pub fn with_memory(
        self,
        recent_window: usize,
        key_turns_cap: usize,
        summary_max_tokens: u32,
        summary_model: String,
    ) -> Self {
        let _ = self.cmd_tx.send(Cmd::SetMemory {
            recent_window,
            key_turns_cap,
            summary_max_tokens,
            summary_model,
        });
        self
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

    /// Заменить ContextBuilder (персона/промпты конкретной встречи).
    pub fn set_ctx(&self, ctx: ContextBuilder) {
        let _ = self.cmd_tx.send(Cmd::SetCtx(ctx));
    }

    /// Сменить модель/уровень рассуждений на живом клиенте.
    pub fn set_llm(&self, model: String, effort: Option<String>) {
        let _ = self.cmd_tx.send(Cmd::SetLlm { model, effort });
    }

    /// Cancel the current generation. keep=true persists the partial text as an answer.
    pub fn cancel(&self, keep: bool) {
        let _ = self.cmd_tx.send(Cmd::Cancel { keep });
    }

    /// Behavior when a new request arrives while generating:
    /// on_resend=true aborts the current run and starts a new one;
    /// keep=true saves the partial text of an aborted run.
    pub fn set_cancel_policy(&self, on_resend: bool, keep: bool) {
        let _ = self.cmd_tx.send(Cmd::SetCancelPolicy { on_resend, keep });
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
                let llm = self.llm.clone();
                let tx = self.cmd_tx.clone();                let model = self.summary_model.clone();
                let max_tok = self.summary_max_tokens;
                let window = self.recent_window;
                let key_cap = self.key_cap;
                let chat_id = self.active.clone();
                {
                    let st = self.active_state();
                    st.turns.push(turn.clone());
                    if is_key_turn(&turn) {
                        st.key_turns.push(turn.clone());
                        if st.key_turns.len() > key_cap {
                            st.key_turns.remove(0);
                        }
                    }
                    if st.turns.len() > window {
                        let drain_n = st.turns.len() - window;
                        let drained: Vec<Turn> = st.turns.drain(..drain_n).collect();
                        let current = st.summary.clone();
                        tokio::spawn(async move {
                            if let Some(s) =
                                summarize(&llm, &model, &current, &drained, max_tok).await
                            {
                                let _ = tx.send(Cmd::SummaryDone(s));
                            }
                        });
                    }
                }
                self.maybe_persist(&chat_id);
                let gen = self.gen.load(Ordering::SeqCst);
                let _ = self.events.send(OrchEvent::Status {
                    gen,
                    state: "listening".into(),
                });
                if auto && turn.speaker == Speaker::Interviewer {
                    self.fire(None, None, inner, false).await;
                }
            }
            Cmd::Partial(text) => self.last_partial = text,
            Cmd::Manual(note, image) => self.fire(note, image, inner, true).await,
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
            Cmd::Cancel { keep } => {
                self.abort_current(keep).await;
            }
            Cmd::SetCancelPolicy { on_resend, keep } => {
                self.cancel_on_resend = on_resend;
                self.cancel_keep = keep;
            }
            Cmd::SummaryDone(s) => {
                if let Some(st) = self.chats.get_mut(&self.active) {
                    st.summary = s;
                }
            }
            Cmd::SetCtx(ctx) => {
                self.ctx = ctx;
            }
            Cmd::SetLlm { model, effort } => {
                self.llm.set_model(model, effort);
            }
            Cmd::SetMemory {
                recent_window,
                key_turns_cap,
                summary_max_tokens,
                summary_model,
            } => {
                self.recent_window = recent_window.max(1);
                self.key_cap = key_turns_cap.max(1);
                self.summary_max_tokens = summary_max_tokens.max(1);
                self.summary_model = summary_model;
            }
        }
    }

    /// Abort the current generation (if any). keep=true: persist the buffered
    /// partial text as a Candidate turn and emit Done.
    async fn abort_current(&mut self, keep: bool) {
        if let Some(a) = self.answer.take() {
            let text = if keep {
                self.gen_buf
                    .as_ref()
                    .and_then(|b| b.try_lock().map(|g| g.clone()).ok())
                    .unwrap_or_default()
            } else {
                String::new()
            };
            self.gen_buf = None;
            a.abort();
            let gen = self.cur_gen;
            if keep && !text.trim().is_empty() {
                let chat_id = self.active.clone();
                let now = chrono::Utc::now();
                let st = self.chats.entry(chat_id.clone()).or_insert_with(ChatState::new);
                st.turns.push(Turn {
                    speaker: Speaker::Candidate,
                    text: text.clone(),
                    start_time: now,
                    end_time: now,
                    typed: false,
                });
                if st.turns.len() > 12 {
                    st.turns.remove(0);
                }
                self.maybe_persist(&chat_id);
                let _ = self.events.send(OrchEvent::Done { gen, text });
            } else {
                let _ = self.events.send(OrchEvent::Status {
                    gen,
                    state: "cancelled".into(),
                });
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
        manual: bool,
    ) {
        if self.answer.is_some() && !self.cancel_on_resend {
            // Повторная отправка не отменяет текущую генерацию — новый запрос отклоняется.
            let gen = self.gen.load(Ordering::SeqCst);
            let _ = self.events.send(OrchEvent::Status {
                gen,
                state: "busy".into(),
            });
            return;
        }
        self.abort_current(self.cancel_keep).await;

        let gen = self.gen.fetch_add(1, Ordering::SeqCst) + 1;
        let chat_id = self.active.clone();
        let (turns, key_turns, summary) = {
            let st = self.active_state();
            (st.turns.clone(), st.key_turns.clone(), st.summary.clone())
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
                typed: false,
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
                    typed: true,
                });
                if st.turns.len() > 12 {
                    st.turns.remove(0);
                }
                self.maybe_persist(&chat_id);
            }
        }

        let input = ContextInput {
            summary: &summary,
            key_turns: &key_turns,
            recent: &turns,
            focus: focus.as_ref(),
            focus_live: focus_was_partial,
            note: note.as_deref(),
            image_b64: image_b64.as_deref(),
            manual,
        };
        let messages = self.ctx.build(&input);
        let (mut rx, handle) = self.llm.stream_answer(messages);
        self.answer = Some(handle);
        self.cur_gen = gen;
        let buf = Arc::new(Mutex::new(String::new()));
        self.gen_buf = Some(buf.clone());
        let events = self.events.clone();
        let inner2 = inner.clone();
        let chat_id2 = chat_id.clone();

        tokio::spawn(async move {
            while let Some(ev) = rx.recv().await {
                match ev {
                    AnswerEvent::Token(t) => {
                        if let Ok(mut b) = buf.try_lock() {
                            b.push_str(&t);
                        }
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
                                    typed: false,
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

/// Асинхронное сжатие current_summary + drained в 2-4 предложения (028).
async fn summarize(
    llm: &LlmClient,
    model: &str,
    current: &str,
    drained: &[Turn],
    max_tokens: u32,
) -> Option<String> {
    fn tag(t: &Turn) -> &'static str {
        match t.speaker {
            Speaker::Interviewer => "I",
            Speaker::Candidate => "C",
        }
    }
    let hist = drained
        .iter()
        .map(|t| format!("{}: {}", tag(t), t.text))
        .collect::<Vec<_>>()
        .join("\n");
    let msgs = vec![
        ChatMessage {
            role: Role::System,
            content: MessageContent::Text(
                "Ты сжимаешь историю технического собеседования в краткое резюме. \
                 Сохрани: обсуждённые темы, заданные вопросы, данные ответы, имена, числа, выводы. \
                 Выведи 2-4 предложения без вступлений."
                    .into(),
            ),
        },
        ChatMessage {
            role: Role::User,
            content: MessageContent::Text(format!(
                "Текущее резюме: {}\n\nНовые реплики:\n{}",
                current, hist
            )),
        },
    ];
    if model.is_empty() {
        llm.complete(msgs, max_tokens, 0.0).await.ok()
    } else {
        llm.complete_with(model, msgs, max_tokens, 0.0).await.ok()
    }
}
