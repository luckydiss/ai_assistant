# Design: Context Builder

## 1. Cargo.toml

```toml
[package]
name = "engine-context"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
engine-dialogue = { path = "../engine-dialogue" }
```

## 2. src/lib.rs

```rust
//! LLM context assembly with token budget
#![deny(clippy::all)]

mod builder;
mod tokens;

pub use builder::*;
pub use tokens::*;
```

## 3. src/tokens.rs

```rust
/// Грубая оценка токенов без внешних библиотек: ~4 символа на токен.
pub fn estimate_tokens(s: &str) -> usize {
    s.chars().count() / 4 + 1
}
```

## 4. src/builder.rs

```rust
use engine_dialogue::{Speaker, Turn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

pub struct ContextBuilder {
    system: String,
    persona: String,
    max_tokens: usize,
}

impl ContextBuilder {
    pub fn new(system: String, persona: String, max_tokens: usize) -> Self {
        Self { system, persona, max_tokens }
    }

    pub fn build(
        &self,
        summary: &str,
        turns: &[Turn],
        focus: &Turn,
        note: Option<&str>,
    ) -> Vec<ChatMessage> {
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
            kept.remove(0); // усекать старейшие
        }
    }

    fn user_content(
        &self,
        summary: &str,
        turns: &[Turn],
        focus: &Turn,
        note: Option<&str>,
    ) -> String {
        let mut s = String::new();
        if !summary.is_empty() {
            s.push_str(&format!("Раньше: {}\n", summary));
        }
        s.push_str("Диалог:\n");
        for t in turns {
            let tag = match t.speaker {
                Speaker::Interviewer => "I",
                Speaker::Candidate => "C",
            };
            s.push_str(&format!("{}: {}\n", tag, t.text));
        }
        s.push_str(&format!("Последний вопрос I: «{}»", focus.text));
        if let Some(n) = note {
            s.push_str(&format!("\nКомментарий пользователя: {}", n));
        }
        s.push_str("\nОтвет по протоколу.");
        s
    }
}
```

## Рассмотрено и отклонено
- **tiktoken-rs:** отклонено — требует загрузки BPE-файлов, у агента нет интернета
- **Отдельный message на каждый turn:** отклонено — один user-блок дешевле по токенам
