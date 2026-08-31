use serde::{Deserialize, Serialize};

/// Человекочитаемые метаданные чат-модели.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    pub family: String,
    /// 0 = неизвестно (провайдер не отдаёт).
    pub context_length: u64,
    pub pricing: Pricing,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Pricing {
    /// USD за 1M input-токенов; 0.0 = неизвестно.
    pub input_per_1m: f64,
    /// USD за 1M output-токенов; 0.0 = неизвестно.
    pub output_per_1m: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Capabilities {
    pub chat: bool,
    pub vision: bool,
    pub tools: bool,
    pub reasoning: bool,
}

impl ModelMetadata {
    pub fn is_chat(&self) -> bool {
        self.capabilities.chat
    }
}

/// Семейство по id/name — единый fallback на весь проект
/// (живёт здесь, а не в UI).
pub fn extract_family(id: &str, name: &str) -> String {
    let lower = format!("{} {}", id, name).to_lowercase();
    for (key, family) in [
        ("claude", "Anthropic"),
        ("gpt", "OpenAI"),
        ("chatgpt", "OpenAI"),
        ("o1", "OpenAI"),
        ("o3", "OpenAI"),
        ("gemini", "Google"),
        ("gemma", "Google"),
        ("llama", "Meta"),
        ("mistral", "Mistral"),
        ("mixtral", "Mistral"),
        ("deepseek", "DeepSeek"),
        ("qwen", "Qwen"),
        ("grok", "xAI"),
        ("glm", "Zhipu"),
        ("kimi", "Moonshot"),
        ("minimax", "MiniMax"),
        ("command", "Cohere"),
        ("step", "Step"),
    ] {
        if lower.contains(key) {
            return family.into();
        }
    }
    "Other".into()
}
