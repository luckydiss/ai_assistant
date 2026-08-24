# Design: Config System

## 1. Config Structure

**crates/engine-config/Cargo.toml:**

```toml
[package]
name = "engine-config"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
toml = "0.8"
notify.workspace = true
tokio.workspace = true
tokio-stream.workspace = true
```

**crates/engine-config/src/lib.rs:**

```rust
//! Configuration management with hot-reload
#![deny(clippy::all)]

mod config;
mod error;
mod watcher;

pub use config::*;
pub use error::*;
pub use watcher::*;
```

**crates/engine-config/src/config.rs:**

```rust
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub stt: SttConfig,
    pub llm: LlmConfig,
    pub vad: VadConfig,
    pub orchestrator: OrchestratorConfig,
    pub prompts: PromptsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SttConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    #[serde(default = "default_chunk_ms")]
    pub chunk_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VadConfig {
    #[serde(default = "default_silence_ms")]
    pub silence_ms: u64,
    #[serde(default = "default_max_segment_ms")]
    pub max_segment_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrchestratorConfig {
    #[serde(default = "default_min_words")]
    pub min_words: usize,
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptsConfig {
    pub system: String,
    pub persona: String,
}

fn default_chunk_ms() -> u64 { 7000 }
fn default_temperature() -> f32 { 0.4 }
fn default_max_tokens() -> u32 { 700 }
fn default_silence_ms() -> u64 { 600 }
fn default_max_segment_ms() -> u64 { 7000 }
fn default_min_words() -> usize { 4 }
fn default_debounce_ms() -> u64 { 600 }

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, crate::ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), crate::ConfigError> {
        if self.vad.silence_ms == 0 {
            return Err(crate::ConfigError::Validation("silence_ms must be > 0".into()));
        }
        if self.orchestrator.min_words == 0 {
            return Err(crate::ConfigError::Validation("min_words must be > 0".into()));
        }
        if !(0.0..=2.0).contains(&self.llm.temperature) {
            return Err(crate::ConfigError::Validation("temperature must be 0.0..=2.0".into()));
        }
        Ok(())
    }
}
```

**crates/engine-config/src/error.rs:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
}
```

**crates/engine-config/src/watcher.rs:**

```rust
use crate::{Config, ConfigError};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    Changed,
    Error(String),
}

pub struct ConfigWatcher {
    config: Arc<RwLock<Config>>,
    sender: broadcast::Sender<ConfigEvent>,
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub fn start<P: AsRef<Path>>(path: P) -> Result<(Arc<RwLock<Config>>, broadcast::Receiver<ConfigEvent>), ConfigError> {
        let config = Config::load(&path)?;
        let config = Arc::new(RwLock::new(config));
        let (sender, _) = broadcast::channel(16);
        let sender_clone = sender.clone();
        let config_clone = config.clone();
        let path = path.as_ref().to_path_buf();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    let sender = sender_clone.clone();
                    let config = config_clone.clone();
                    let path = path.clone();
                    tokio::spawn(async move {
                        sleep(Duration::from_millis(100)).await; // debounce
                        match Config::load(&path) {
                            Ok(new_config) => {
                                *config.write().unwrap() = new_config;
                                let _ = sender.send(ConfigEvent::Changed);
                            }
                            Err(e) => {
                                let _ = sender.send(ConfigEvent::Error(e.to_string()));
                            }
                        }
                    });
                }
            }
        })?;

        watcher.watch(path.parent().unwrap(), RecursiveMode::NonRecursive)?;

        Ok((config, sender.subscribe()))
    }
}
```

## 2. Example Config File

**config.toml:**

```toml
[stt]
provider = "groq"
api_key = "gsk_..."
model = "whisper-large-v3-turbo"
chunk_ms = 7000

[llm]
provider = "openai"
api_key = "sk-..."
model = "gpt-4o-mini"
base_url = "https://api.openai.com/v1"
temperature = 0.4
max_tokens = 700

[vad]
silence_ms = 600
max_segment_ms = 7000

[orchestrator]
min_words = 4
debounce_ms = 600

[prompts]
system = """Ты — невидимый ассистент на техсобесе. Даётся диалог: I — интервьюер, C — кандидат.
Помоги кандидату ответить на ПОСЛЕДНИЙ вопрос I.
Протокол: если последняя реплика I — не вопрос к кандидату — верни <SKIP> и остановись.
Иначе сразу суть: 2–5 буллетов или короткий код-блок.
Язык = язык вопроса. Без вступлений."""
persona = "Senior Rust developer, 5 years experience"
```

## 3. Public API

```rust
// Load config once at startup
let config = Config::load("config.toml")?;

// Watch for changes
let (config_lock, mut events) = ConfigWatcher::start("config.toml")?;

// Subscribe to changes
tokio::spawn(async move {
    while let Ok(event) = events.recv().await {
        match event {
            ConfigEvent::Changed => tracing::info!("Config reloaded"),
            ConfigEvent::Error(e) => tracing::error!("Config error: {}", e),
        }
    }
});

// Access current config
let current = config_lock.read().unwrap();
println!("LLM model: {}", current.llm.model);
```

## Рассмотрено и отклонено
- **JSON вместо TOML:** отклонено, TOML более читаем для конфигов
- **Global static config:** отклонено, используем Arc<RwLock> для hot-reload
- **Sync watcher:** отклонено, используем async для интеграции с tokio
