use crate::error::ModelsError;
use crate::metadata::{extract_family, Capabilities, ModelMetadata, Pricing};
use crate::provider::ModelProvider;

/// Стоп-слова: id с этими подстроками — не чатовые модели
/// (перенесено из commands.rs; применяется к plain OpenAI-формату,
/// где нет явных capabilities).
const NON_CHAT: &[&str] = &[
    "tts", "image", "embed", "rerank", "music", "ocr", "whisper", "moderation", "veo",
    "kling", "seedance", "recraft", "krea", "sakana", "fugu", "inkling", "hy3", "gte",
    "nano-banana", "transcribe", "video",
];

/// Generic OpenAI-compatible каталог моделей: работает и с plain
/// OpenAI-форматом (только id), и с rich-форматом OpenRouter
/// (pricing per-token, context_length, architecture.modality).
pub struct OpenAiCompatCatalog {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl OpenAiCompatCatalog {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent("curl/8.0")
                .build()
                .unwrap_or_default(),
            base_url,
            api_key,
        }
    }

    pub async fn fetch_raw(&self) -> Result<serde_json::Value, ModelsError> {
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let resp = self.http.get(&url).bearer_auth(&self.api_key).send().await?;
        if !resp.status().is_success() {
            return Err(ModelsError::Http(resp.status().as_u16()));
        }
        let v: serde_json::Value = resp.json().await?;
        if v["data"].as_array().is_none() {
            return Err(ModelsError::InvalidResponse);
        }
        Ok(v)
    }

    fn parse_one(m: &serde_json::Value) -> Option<ModelMetadata> {
        let id = m["id"].as_str()?.to_string();
        let name = m["name"]
            .as_str()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| id.clone());
        let family = extract_family(&id, &name);

        // Rich-формат OpenRouter.
        let ctx = m["context_length"].as_u64().unwrap_or(0);
        let modality = m["architecture"]["modality"].as_str().unwrap_or("");
        let prompt_price = m["pricing"]["prompt"].as_str();
        let completion_price = m["pricing"]["completion"].as_str();

        let lower_id = id.to_lowercase();
        let reasoning = ["o1", "o3", "o4", "r1"].iter().any(|k| lower_id.contains(k))
            || lower_id.contains("reasoning")
            || lower_id.contains("thinking")
            || name.to_lowercase().contains("reasoning");

        let (pricing, chat) = if prompt_price.is_some() || completion_price.is_some() {
            // Rich: провайдер отдаёт pricing → считаем модель чатовой.
            let p = Pricing {
                input_per_1m: prompt_price
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0)
                    * 1_000_000.0,
                output_per_1m: completion_price
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0)
                    * 1_000_000.0,
            };
            (p, true)
        } else {
            // Plain:capabilities — по эвристике над id.
            (
                Pricing::default(),
                !NON_CHAT.iter().any(|w| lower_id.contains(w)),
            )
        };

        Some(ModelMetadata {
            id,
            name,
            family,
            context_length: ctx,
            pricing,
            capabilities: Capabilities {
                chat,
                vision: modality.contains("image"),
                tools: chat,
                reasoning,
            },
        })
    }
}

#[async_trait::async_trait]
impl ModelProvider for OpenAiCompatCatalog {
    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn api_key(&self) -> &str {
        &self.api_key
    }

    async fn list_models(&self) -> Result<Vec<ModelMetadata>, ModelsError> {
        let v = self.fetch_raw().await?;
        Ok(v["data"]
            .as_array()
            .expect("checked in fetch_raw")
            .iter()
            .filter_map(Self::parse_one)
            .collect())
    }

    async fn validate_model(&self, id: &str) -> Result<(), ModelsError> {
        match self.list_models().await {
            Ok(models) => {
                if models.iter().any(|m| m.id == id) {
                    Ok(())
                } else {
                    Err(ModelsError::UnknownModel(id.to_string()))
                }
            }
            // Каталог недоступен → не блокируем работу.
            Err(e) => {
                tracing::warn!(error = %e, "model catalog unavailable, skipping validation");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RICH_JSON: &str = r#"{"data":[
        {"id":"anthropic/claude-3.5-sonnet","name":"Claude 3.5 Sonnet",
         "context_length":200000,"pricing":{"prompt":"0.000003","completion":"0.000015"},
         "architecture":{"modality":"text+image->text"}},
        {"id":"openai/gpt-4o-mini","name":"GPT-4o mini",
         "context_length":128000,"pricing":{"prompt":"0.00000015","completion":"0.0000006"},
         "architecture":{"modality":"text->text"}}
    ]}"#;

    const PLAIN_JSON: &str = r#"{"data":[
        {"id":"gpt-5.6-luna","object":"model"},
        {"id":"tts-1","object":"model"},
        {"id":"text-embedding-3-small","object":"model"},
        {"id":"claude-sonnet-4-5"}
    ]}"#;

    fn parse_all(body: &str) -> Vec<ModelMetadata> {
        serde_json::from_str::<serde_json::Value>(body)
            .unwrap()["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(OpenAiCompatCatalog::parse_one)
            .collect()
    }

    #[test]
    fn model_metadata_fields() {
        let ms = parse_all(RICH_JSON);
        assert_eq!(ms.len(), 2);
        let c = &ms[0];
        assert_eq!(c.id, "anthropic/claude-3.5-sonnet");
        assert_eq!(c.name, "Claude 3.5 Sonnet");
        assert_eq!(c.family, "Anthropic");
        assert_eq!(c.context_length, 200_000);
        assert!((c.pricing.input_per_1m - 3.0).abs() < 1e-9);
        assert!((c.pricing.output_per_1m - 15.0).abs() < 1e-9);
        assert!(c.capabilities.chat);
        assert!(c.capabilities.vision);
        assert!(c.capabilities.tools);
    }

    #[test]
    fn metadata_openai_fallback() {
        let ms = parse_all(PLAIN_JSON);
        assert_eq!(ms.len(), 4);
        // Всё деградировало без паники: дефолтные pricing/ctx.
        for m in &ms {
            assert_eq!(m.pricing, Pricing::default());
            assert_eq!(m.context_length, 0);
        }
        // Стоп-слова отфильтрованы из чатовых.
        let chat: Vec<_> = ms.iter().filter(|m| m.is_chat()).collect();
        let ids: Vec<_> = chat.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, vec!["gpt-5.6-luna", "claude-sonnet-4-5"]);
        assert!(ms[0].family == "OpenAI");
        assert!(ms[3].family == "Anthropic");
    }

    #[test]
    fn filter_chat_only() {
        let ms = parse_all(RICH_JSON);
        // Rich-формат: всё чатовое, vision по modality.
        assert!(ms.iter().all(|m| m.is_chat()));
        let plain = parse_all(PLAIN_JSON);
        assert_eq!(
            plain.iter().filter(|m| m.is_chat()).count(),
            2,
            "tts/embed отфильтрованы"
        );
    }

    #[test]
    fn family_from_metadata() {
        assert_eq!(extract_family("anthropic/claude-3.5-sonnet", ""), "Anthropic");
        assert_eq!(extract_family("google/gemini-2.0-flash", "Gemini"), "Google");
        assert_eq!(extract_family("qwen/qwen3-max", ""), "Qwen");
        assert_eq!(extract_family("totally-unknown-model", ""), "Other");
    }

    /// Mock TCP server, как в engine-llm tests: сырой HTTP/1.1 ответ на GET.
    async fn spawn_mock(body: &'static str, status_line: &'static str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            // Несколько последовательных запросов.
            while let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = [0u8; 4096];
                let _ = sock.read(&mut buf).await;
                let resp = format!(
                    "{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status_line,
                    body.len(),
                    body
                );
                let _ = sock.write_all(resp.as_bytes()).await;
                let _ = sock.flush().await;
            }
        });
        format!("http://{}/v1", addr)
    }

    #[tokio::test]
    async fn openrouter_list_models_via_mock() {
        let url = spawn_mock(RICH_JSON, "HTTP/1.1 200 OK").await;
        let catalog = OpenAiCompatCatalog::new(url, "key".into());
        let models = catalog.list_models().await.unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].name, "Claude 3.5 Sonnet");
    }

    #[tokio::test]
    async fn validate_unknown_and_http_down() {
        let url = spawn_mock(RICH_JSON, "HTTP/1.1 200 OK").await;
        let catalog = OpenAiCompatCatalog::new(url, "key".into());
        // Существующая.
        assert!(catalog.validate_model("anthropic/claude-3.5-sonnet").await.is_ok());
        // Несуществующая.
        let err = catalog.validate_model("no/such-model").await.unwrap_err();
        assert!(matches!(err, ModelsError::UnknownModel(_)));

        // Каталог недоступен → Ok без исключения.
        let bad = OpenAiCompatCatalog::new("http://127.0.0.1:1/v1".into(), "key".into());
        assert!(bad.validate_model("whatever").await.is_ok());
    }
}
