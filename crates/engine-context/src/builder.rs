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
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<Part>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: MessageContent,
}

pub struct ContextBuilder {
    system: String,
    manual_system: String,
    persona: String,
    max_tokens: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    pub base_system: String,
    pub role: String,
    pub extra_prompt: String,
    pub resume_text: String,
    pub vacancy: String,
}

/// Структурированный вход контекста: трёхслойная память + фокус + заметка.
pub struct ContextInput<'a> {
    pub summary: &'a str,
    pub key_turns: &'a [Turn],
    pub recent: &'a [Turn],
    pub focus: Option<&'a Turn>,
    /// focus — живой partial (интервьюер ещё говорит).
    pub focus_live: bool,
    pub note: Option<&'a str>,
    pub image_b64: Option<&'a str>,
    /// Ручной запрос (хоткей/текст/скриншот) — использовать manual_system.
    pub manual: bool,
}

impl<'a> ContextInput<'a> {
    pub fn new(recent: &'a [Turn]) -> Self {
        Self {
            summary: "",
            key_turns: &[],
            recent,
            focus: None,
            focus_live: false,
            note: None,
            image_b64: None,
            manual: false,
        }
    }
}

impl ContextBuilder {
    pub fn new(system: String, persona: String, max_tokens: usize) -> Self {
        Self {
            system,
            manual_system: String::new(),
            persona,
            max_tokens,
        }
    }

    /// Отдельный системный промпт для ручных запросов (пусто → system).
    pub fn with_manual_system(mut self, manual_system: String) -> Self {
        self.manual_system = manual_system;
        self
    }

    pub fn with_workspace(base_system: String, ws: &PromptContext, max_tokens: usize) -> Self {
        let mut system = base_system;
        if !ws.extra_prompt.is_empty() {
            system.push_str(&format!("\n\n{}", ws.extra_prompt));
        }
        let mut persona = ws.role.clone();
        if !ws.resume_text.is_empty() {
            persona.push_str(&format!("\nРезюме кандидата: {}", ws.resume_text));
        }
        if !ws.vacancy.is_empty() {
            persona.push_str(&format!("\nВакансия: {}", ws.vacancy));
        }
        Self {
            system,
            manual_system: String::new(),
            persona,
            max_tokens,
        }
    }

    pub fn build(&self, inp: &ContextInput) -> Vec<ChatMessage> {
        let base = if inp.manual && !self.manual_system.is_empty() {
            &self.manual_system
        } else {
            &self.system
        };
        let system = format!("{}\n\nPersona кандидата: {}", base, self.persona);
        let mut recent: Vec<Turn> = inp.recent.to_vec();

        loop {
            let user = self.user_content(inp, &recent);
            let total = crate::estimate_tokens(&system) + crate::estimate_tokens(&user);
            if total <= self.max_tokens || recent.is_empty() {
                return vec![
                    ChatMessage {
                        role: Role::System,
                        content: MessageContent::Text(system),
                    },
                    ChatMessage {
                        role: Role::User,
                        content: self.user_content_message(user, inp.image_b64),
                    },
                ];
            }
            recent.remove(0); // страховка бюджета: усекать старейшие recent
        }
    }

    fn user_content_message(&self, text: String, image_b64: Option<&str>) -> MessageContent {
        match image_b64 {
            Some(img) => MessageContent::Parts(vec![
                Part {
                    kind: "image_url".into(),
                    text: None,
                    image_url: Some(ImageUrl {
                        url: format!("data:image/png;base64,{}", img),
                    }),
                },
                Part {
                    kind: "text".into(),
                    text: Some(text),
                    image_url: None,
                },
            ]),
            None => MessageContent::Text(text),
        }
    }

    fn tag(t: &Turn) -> &'static str {
        match t.speaker {
            Speaker::Interviewer => "I",
            Speaker::Candidate => "C",
        }
    }

    fn user_content(&self, inp: &ContextInput, recent: &[Turn]) -> String {
        let mut s = String::new();
        if !inp.summary.is_empty() {
            s.push_str(&format!("Резюме всей сессии: {}\n", inp.summary));
        }
        if !inp.key_turns.is_empty() {
            s.push_str("Ключевые моменты:\n");
            for t in inp.key_turns {
                s.push_str(&format!("{}: {}\n", Self::tag(t), t.text));
            }
        }
        s.push_str("Недавние реплики:\n");
        for t in recent {
            s.push_str(&format!("{}: {}\n", Self::tag(t), t.text));
        }
        if let Some(f) = inp.focus {
            let live = if inp.focus_live { " (ещё говорит)" } else { "" };
            s.push_str(&format!("Последний вопрос I{live}: «{}»\n", f.text));
        }
        if let Some(n) = inp.note {
            s.push_str(&format!("Комментарий пользователя: {}\n", n));
        }
        s.push_str("Ответь по запросу кандидата.");
        s
    }
}
