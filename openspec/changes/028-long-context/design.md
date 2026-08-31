# Design: Long-Context Memory

## 1. Config (engine-config)

```rust
#[serde(default)] pub context: ContextSection,
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextSection {
    #[serde(default = "def_recent")] pub recent_window: usize,        // 12
    #[serde(default = "def_keycap")] pub key_turns_cap: usize,        // 12
    #[serde(default = "def_sumtok")] pub summary_max_tokens: u32,     // 300
    #[serde(default)] pub summary_model: String,                     // "" → llm.model
}
// validate(): recent_window>0, key_turns_cap>0
```

## 2. engine-context: ContextInput + build

```rust
pub struct ContextInput<'a> {
    pub summary: &'a str,
    pub key_turns: &'a [Turn],
    pub recent: &'a [Turn],
    pub focus: Option<&'a Turn>,
    pub note: Option<&'a str>,
}

impl ContextBuilder {
    pub fn build(&self, inp: &ContextInput) -> Vec<ChatMessage> {
        let system = format!("{}\n\nPersona кандидата: {}", self.system, self.persona);
        let mut recent: Vec<Turn> = inp.recent.to_vec();
        loop {
            let user = self.user_content(inp, &recent);
            let total = crate::estimate_tokens(&system) + crate::estimate_tokens(&user);
            if total <= self.max_tokens || recent.is_empty() {
                return vec![
                    ChatMessage { role: Role::System, content: system },
                    ChatMessage { role: Role::User, content: user },
                ];
            }
            recent.remove(0); // страховка: усекать старейшие recent
        }
    }

    fn tag(t: &Turn) -> &'static str { match t.speaker { Speaker::Interviewer => "I", _ => "C" } }

    fn user_content(&self, inp: &ContextInput, recent: &[Turn]) -> String {
        let mut s = String::new();
        if !inp.summary.is_empty() { s.push_str(&format!("Резюме всей сессии: {}\n", inp.summary)); }
        if !inp.key_turns.is_empty() {
            s.push_str("Ключевые моменты:\n");
            for t in inp.key_turns { s.push_str(&format!("{}: {}\n", Self::tag(t), t.text)); }
        }
        s.push_str("Недавние реплики:\n");
        for t in recent { s.push_str(&format!("{}: {}\n", Self::tag(t), t.text)); }
        if let Some(f) = inp.focus { s.push_str(&format!("Последний вопрос I: «{}»\n", f.text)); }
        if let Some(n) = inp.note { s.push_str(&format!("Комментарий пользователя: {}\n", n)); }
        s.push_str("Ответь по запросу кандидата.");
        s
    }
}
```

## 3. engine-orchestrator: память

ChatState += `key_turns: Vec<Turn>`.

```rust
// чистая функция, тестируема
pub fn is_key_turn(t: &Turn) -> bool {
    let text = t.text.to_lowercase();
    let markers = ["напиши","объясни","расскажи","как работает","почему","что будет",
                   "код","сложность","нарисуй","спроектируй"];
    markers.iter().any(|m| text.contains(m)) || t.text.chars().count() > 200
}

// в handle(Cmd::Turn): после push в turns:
if is_key_turn(&turn) {
    st.key_turns.push(turn.clone());
    if st.key_turns.len() > self.key_cap { st.key_turns.remove(0); }
}
if st.turns.len() > self.recent_window {
    let drain_n = st.turns.len() - self.recent_window;
    let drained: Vec<Turn> = st.turns.drain(..drain_n).collect();
    let current = st.summary.clone();
    let llm = self.llm.clone();
    let tx = self.cmd_tx.clone();
    tokio::spawn(async move {
        if let Some(s) = summarize(&llm, &current, &drained).await {
            let _ = tx.send(Cmd::SummaryDone(s));
        }
    });
}

// новый variant + handle:
enum Cmd { ..., SummaryDone(String) }
Cmd::SummaryDone(s) => { if let Some(st) = self.chats.get_mut(&self.active) { st.summary = s; } }

async fn summarize(llm: &LlmClient, current: &str, drained: &[Turn]) -> Option<String> {
    let hist: String = drained.iter()
        .map(|t| format!("{}: {}", match t.speaker { Speaker::Interviewer => "I", _ => "C" }, t.text))
        .collect::<Vec<_>>().join("\n");
    let msgs = vec![
        ChatMessage { role: Role::System, content:
            "Ты сжимаешь историю технического собеседования в краткое резюме. \
             Сохрани: обсуждённые темы, заданные вопросы, данные ответы, имена, числа, выводы. \
             Выведи 2-4 предложения без вступлений.".into() },
        ChatMessage { role: Role::User, content: format!("Текущее резюме: {}\n\nНовые реплики:\n{}", current, hist) },
    ];
    llm.complete(msgs, 300, 0.0).await.ok()
}
```

## 4. fire(): собирать ContextInput

```rust
let st = self.chats.get(&self.active);
let input = ContextInput {
    summary: st.map(|s| s.summary.as_str()).unwrap_or(""),
    key_turns: st.map(|s| s.key_turns.as_slice()).unwrap_or(&[]),
    recent: st.map(|s| s.turns.as_slice()).unwrap_or(&[]),
    focus: focus.as_ref(),
    note: note.as_deref(),
};
let messages = self.ctx.build(&input);
```

## 5. Тесты

- engine-context: builds_all_layers, skips_empty_layers, budget_safety, context_input_fields; обновить старые вызовы build.
- engine-orchestrator: key_question_detected, short_not_key, key_turns_cap, recent_window_drain, summary_updates (mock complete из 008/026).

## Рассмотрено и отклонено
- **RAG по истории:** отклонено (оверинжиниринг)
- **Суммаризация в реальном времени:** отклонено (только по границе окна)
