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

impl ContextBuilder {
    pub fn new(system: String, persona: String, max_tokens: usize) -> Self {
        Self {
            system,
            persona,
            max_tokens,
        }
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
            persona,
            max_tokens,
        }
    }

    pub fn build(
        &self,
        summary: &str,
        turns: &[Turn],
        focus: Option<&Turn>,
        focus_live: bool,
        note: Option<&str>,
        image_b64: Option<&str>,
    ) -> Vec<ChatMessage> {
        let system = format!("{}\n\nPersona кандидата: {}", self.system, self.persona);
        let mut kept = turns.to_vec();

        loop {
            let user = self.user_content(summary, &kept, focus, focus_live, note);
            let total = crate::estimate_tokens(&system) + crate::estimate_tokens(&user);
            if total <= self.max_tokens || kept.is_empty() {
                return vec![
                    ChatMessage {
                        role: Role::System,
                        content: MessageContent::Text(system),
                    },
                    ChatMessage {
                        role: Role::User,
                        content: self.user_content_message(user, image_b64),
                    },
                ];
            }
            kept.remove(0);
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

    fn user_content(
        &self,
        summary: &str,
        turns: &[Turn],
        focus: Option<&Turn>,
        focus_live: bool,
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
        if let Some(f) = focus {
            let live = if focus_live { " (ещё говорит)" } else { "" };
            s.push_str(&format!("Последний вопрос I{live}: «{}»\n", f.text));
        }
        if let Some(n) = note {
            s.push_str(&format!("Комментарий пользователя: {}\n", n));
        }
        s.push_str("Ответь по запросу кандидата.");
        s
    }
}
