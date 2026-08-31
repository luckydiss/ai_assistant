# Design: Unified Model Providers

## 0. Отклонения от исходного дизайна (обязательные)

1. **Миграция НЕ заменяет credentials.** Ключ dslab.tech не действителен на
   openrouter.ai — авто-подмена сломала бы работающее приложение (STOP Protocol).
   Легаси `llm.base_url`+`llm.api_key` синтезируются в `[providers.<host>]`,
   OpenRouter добавляется пользователем вручную.
2. **Один generic provider** вместо отдельного OpenRouterProvider: оба endpoint'а
   — OpenAI-compatible `/chat/completions`; rich-поля OpenRouter
   (pricing/context_length/architecture) парсятся опционально.
3. **Валидация модели при llm_set**, а не перед каждым запросом (latency).

## 1. engine-models crate

```
crates/engine-models/src/
├── lib.rs
├── provider.rs      // trait ModelProvider
├── catalog.rs       // OpenAiCompatCatalog: list/validate/parse
├── metadata.rs      // ModelMetadata, Pricing, Capabilities
└── error.rs
```

### Cargo.toml

```toml
[package]
name = "engine-models"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
async-trait.workspace = true
reqwest.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio.workspace = true
tracing.workspace = true

[dev-dependencies]
tokio = { workspace = true }
```

### workspace Cargo.toml: + async-trait = "0.1"

## 2. ModelMetadata (metadata.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,              // "anthropic/claude-3.5-sonnet" | "gpt-5.6-luna"
    pub name: String,            // человекочитаемое; fallback → id
    pub family: String,          // "Anthropic"; fallback extract_family(id)
    pub context_length: u64,     // 200000; 0 если неизвестно
    pub pricing: Pricing,        // 0.0 если неизвестно
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub struct Pricing {
    /// USD за 1M input-токенов
    pub input_per_1m: f64,
    /// USD за 1M output-токенов
    pub output_per_1m: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Capabilities {
    pub chat: bool,
    pub vision: bool,
    pub tools: bool,
    pub reasoning: bool,
}
```

Capabilities для chat: rich-модель OpenRouter — modality/image; plain OpenAI —
id-heuristic (единый список стоп-слов, перенесённый из commands.rs):
НЕ chat: tts|image|embed|rerank|music|ocr|whisper|moderation|veo|kling|
seedance|recraft|krea|sakana|fugu|inkling|hy3|gte|nano-banana|transcribe|video.

## 3. ModelProvider trait (provider.rs)

```rust
#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
    fn base_url(&self) -> &str;
    fn api_key(&self) -> &str;
    async fn list_models(&self) -> Result<Vec<ModelMetadata>, ModelsError>;
    /// Валидация id по каталогу. Незнакомый каталог (http != success) → Ok(())
    /// чтобы не блокировать работу при недоступности /models.
    async fn validate_model(&self, id: &str) -> Result<(), ModelsError>;
}
```

## 4. OpenAiCompatCatalog (catalog.rs)

Конструктор `new(base_url, api_key)`. Парсинг JSON:

```rust
// OpenAI format:  {"data":[{"id": "gpt-x", ...}]}
// OpenRouter доп.: per-token pricing ("0.000003"), context_length,
//                  architecture.modality ("text->text" | "text+image->text")
let raw = v["data"].as_array().ok_or(InvalidResponse)?;
for m in raw {
    let Some(id) = m["id"].as_str() else { continue };
    let vision = m["architecture"]["modality"]
        .as_str().map(|s| s.contains("image")).unwrap_or(false);
    let ctx = m["context_length"].as_u64().unwrap_or(0);
    let prompt = m["pricing"]["prompt"].as_str()
        .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) * 1e6;
    // ...
    let meta = if m["pricing"]["prompt"].is_string() || m["context_length"].is_u64() {
        // rich: capabilities.chat = true (OpenRouter отдаёт только чатовые)
        ...
    } else {
        // plain: chat = !STOP_WORDS.iter().any(|w| id.contains(w)),
        //        reasoning = id содержит "o1"|"o3"|"reasoning"|"thinking"
        ...
    };
}
```

`validate_model`: fetch list_models, `any(|m| m.id == id)` → Ok / Err(UnknownModel);
если HTTP упал — Ok(()) с tracing::warn! (не блокируем чат из-за каталога).

## 5. Provider Registry (config.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default)] pub api_key: String,
    #[serde(default = "def_openrouter_url")] pub base_url: String,
    #[serde(default = "def_true")] pub enabled: bool,
}
fn def_openrouter_url() -> String { "https://openrouter.ai/api/v1".into() }
fn def_true() -> bool { true }

pub struct Config {
    ...,
    #[serde(default)] pub providers: BTreeMap<String, ProviderConfig>,
}

impl Config {
    /// Легаси-конверсия БЕЗ порчи рабочей авторизации:
    /// base_url/api_key в [llm] при пустой секции providers →
    /// [providers.<host>], llm.provider = <host>.
    pub fn normalize(&mut self) {
        if self.providers.is_empty()
            && !self.llm.api_key.is_empty()
        {
            let host = self.llm.base_url.as_deref()
                .and_then(|u| url::host_of(u))   // "api.dslab.tech"
                .unwrap_or("custom");
            let key = sanitize(host);             // alnum -> "dslab"
            self.providers.insert(key.clone(), ProviderConfig {
                api_key: self.llm.api_key.clone(),
                base_url: self.llm.base_url.clone().unwrap_or_default(),
                enabled: true,
            });
            self.llm.provider = key;
        }
        if self.llm.provider.is_empty() && !self.providers.is_empty() {
            self.llm.provider = self.providers.keys().next().cloned().unwrap();
        }
        if self.llm.provider.is_empty() { self.llm.provider = "openrouter".into(); }
        self.providers.entry(self.llm.provider.clone())
            .or_insert_with(|| ProviderConfig {
                api_key: self.llm.api_key.clone(),
                base_url: def_openrouter_url(), enabled: true,
            });
    }

    pub fn get_provider(&self) -> Result<engine_models::OpenAiCompatCatalog, crate::ConfigError> {
        match self.providers.get(&self.llm.provider) {
            Some(p) if p.enabled => Ok(engine_models::OpenAiCompatCatalog::new(
                p.base_url.trim_end_matches('/').to_string(), p.api_key.clone())),
            Some(_) => Err(ConfigError::Validation(format!("provider {} disabled", self.llm.provider))),
            None => Err(ConfigError::Validation(format!("provider {} not found", self.llm.provider))),
        }
    }
}
```

validate(): существующее поведение сохраняется; новое правило применяется
только когда `providers` непустое И provider нет в карте → Err.

## 6. engine-llm интеграция (минимальная, без поломки тестов)

LlmClient сигнатура `new(base_url, api_key, ...)` СОХРАНЯЕТСЯ (20 вызовов).
Добавляется опциональный каталог:

```rust
pub struct LlmClient {
    ...,
    catalog: Option<Arc<dyn engine_models::ModelProvider>>,
}

impl LlmClient {
    pub fn with_catalog(mut self, catalog: Arc<dyn ModelProvider>) -> Self;
    /// Err, если модель не найдена в каталоге (каталог недоступен → Ok).
    pub async fn validate_model(&self) -> Result<(), String>;
}
```

commands::llm_set: `services.orch.set_llm(...)` ВЫЗЫВАЕТСЯ только после
успешной validate (fail-fast, живой клиент не ломается несуществующей моделью).

## 7. commands.rs

```rust
#[tauri::command]
pub async fn models_list(cfg: State<'_, ConfigState>) -> Result<Vec<ModelMetadata>, String> {
    let provider = cfg.read().map_err(|e| e.to_string())?.get_provider().map_err(|e| e.to_string())?;
    let all = provider.list_models().await.map_err(|e| e.to_string())?;
    Ok(all.into_iter().filter(|m| m.capabilities.chat).collect())
}
```

Стоп-слова и extract_family переезжают СЮДА ИЗ UI/команды — единый источник.

## 8. UI (overlay.js/html/css)

- models_list теперь объекты: `{id, name, family, context_length, pricing:{input_per_1m,output_per_1m}, capabilities}`.
- Удаляются: `familyOf()` regex-цепочка, EXCLUDE-подход, JS-карта цветов семейств.
- Провайдеры-группы: уникальные `family` в порядке сортировки; первый пункт «все».
- Цвета: CSS-классы `.mm-fam-0..7` (циклическая палитра по index группы), не по имени.
- Карточка модели: name, id, бейджи «{ctx}k» (ctx>0), «$in/out за 1M» (price>0),
  значки caps: vision 👁, tools ⚙, reasoning 💭.
- prettyName() удаляется — приходит name (fallback id) с бэкенда.

## 9. Тесты

- engine-models: model_metadata_fields (rich parse), metadata_openai_fallback
  (plain), filter_chat_only, family_from_metadata, validate_unknown/http-down
  (mock TCP server как в engine-llm tests)
- engine-config: legacy_migration_nonbreaking, openrouter_defaults,
  validates_provider_exists, active_provider_roundtrip
- engine-llm: temperature_retry (сохраняется), поток с каталогом на mock

## Рассмотрено и отклонено

- **Кэширование моделей:** список тянется на каждое открытие модалки (Non-goal).
- **Multi-provider fallback:** один активный провайдер.
- **WebSocket streaming:** SSE совместим с текущим кодом.
