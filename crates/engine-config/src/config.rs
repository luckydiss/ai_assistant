use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Запись реестра провайдеров: [providers.<name>].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "def_openrouter_url")]
    pub base_url: String,
    #[serde(default = "def_true")]
    pub enabled: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: def_openrouter_url(),
            enabled: true,
        }
    }
}

fn def_openrouter_url() -> String {
    "https://openrouter.ai/api/v1".into()
}
fn def_true() -> bool {
    true
}

/// Host → короткое имя провайдера ("api.dslab.tech" → "dslab").
fn provider_name_from_host(url: &str) -> String {
    let host = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(url);
    let host = host.split(['/', ':']).next().unwrap_or("");
    let name = host
        .split('.')
        .find(|l| !matches!(*l, "api" | "www" | "openai"))
        .unwrap_or("custom");
    let clean: String = name.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if clean.is_empty() {
        "custom".into()
    } else {
        clean
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub stt: SttConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub vad: VadConfig,
    #[serde(default)]
    pub prompts: PromptsConfig,
    #[serde(default)]
    pub audio: AudioSection,
    #[serde(default)]
    pub hotkeys: HotkeysSection,
    #[serde(default)]
    pub tts: TtsSection,
    #[serde(default)]
    pub ui: UiSection,
    #[serde(default)]
    pub window: WindowSection,
    #[serde(default)]
    pub chat: ChatSection,
    #[serde(default)]
    pub context: ContextSection,
    /// Реестр провайдеров моделей: [providers.openrouter], [providers.dslab], …
    #[serde(default)]
    pub providers: BTreeMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    pub provider: String,
    pub api_key: String,
    #[serde(default)]
    pub soniox_api_key: String,
    pub model: String,
    #[serde(default = "def_language_hints")]
    pub language_hints: String,
    #[serde(default = "def_stt_language")]
    pub language: String,
    #[serde(default = "def_utterance_idle_ms")]
    pub utterance_idle_ms: u64,
    #[serde(default = "def_max_utterance_chars")]
    pub max_utterance_chars: usize,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            provider: def_stt_provider(),
            api_key: String::new(),
            soniox_api_key: String::new(),
            model: def_stt_model(),
            language_hints: def_language_hints(),
            language: def_stt_language(),
            utterance_idle_ms: def_utterance_idle_ms(),
            max_utterance_chars: def_max_utterance_chars(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default)]
    pub reasoning_effort: Option<String>,
    #[serde(default)]
    pub search_enabled: bool,
    #[serde(default)]
    pub search_tool_json: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: String::new(),
            api_key: String::new(),
            model: String::new(),
            base_url: None,
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            reasoning_effort: None,
            search_enabled: false,
            search_tool_json: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    #[serde(default = "default_silence_ms")]
    pub silence_ms: u64,
    #[serde(default = "default_max_segment_ms")]
    pub max_segment_ms: u64,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            silence_ms: default_silence_ms(),
            max_segment_ms: default_max_segment_ms(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsConfig {
    #[serde(default)]
    pub system: String,
    #[serde(default)]
    pub persona: String,
    /// Отдельный системный промпт для ручных запросов (пусто → system).
    #[serde(default)]
    pub manual_system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSection {
    #[serde(default = "def_source")]
    pub source: String,
    #[serde(default = "def_mode")]
    pub mode: String,
    #[serde(default)]
    pub mic_device: Option<String>,
}

impl Default for AudioSection {
    fn default() -> Self {
        Self {
            source: def_source(),
            mode: def_mode(),
            mic_device: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeysSection {
    #[serde(default = "hk_manual")]
    pub manual: String,
    #[serde(default = "hk_hide")]
    pub hide: String,
    #[serde(default = "hk_click")]
    pub click_through: String,
    #[serde(default = "hk_mute")]
    pub mute: String,
    #[serde(default = "hk_record")]
    pub record: String,
    #[serde(default = "hk_shot")]
    pub screenshot_full: String,
    #[serde(default = "hk_shotw")]
    pub screenshot_region: String,
    #[serde(default = "hk_tts")]
    pub tts: String,
}

impl Default for HotkeysSection {
    fn default() -> Self {
        Self {
            manual: hk_manual(),
            hide: hk_hide(),
            click_through: hk_click(),
            mute: hk_mute(),
            record: hk_record(),
            screenshot_full: hk_shot(),
            screenshot_region: hk_shotw(),
            tts: hk_tts(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsSection {
    #[serde(default = "def_tts_mode")]
    pub mode: String,
    #[serde(default = "def_tts_provider")]
    pub provider: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "def_tts_model")]
    pub model_id: String,
    #[serde(default = "def_tts_voice")]
    pub voice_id: String,
    #[serde(default = "def_tts_rate")]
    pub sample_rate: u32,
}

impl Default for TtsSection {
    fn default() -> Self {
        Self {
            mode: def_tts_mode(),
            provider: def_tts_provider(),
            api_key: String::new(),
            model_id: def_tts_model(),
            voice_id: def_tts_voice(),
            sample_rate: def_tts_rate(),
        }
    }
}

fn def_tts_mode() -> String {
    "off".into()
}
fn def_tts_provider() -> String {
    "cartesia".into()
}
fn def_tts_model() -> String {
    "sonic-3.5".into()
}
fn def_tts_voice() -> String {
    "1e4176b1-3db9-44d6-a601-4fe68b041942".into()
}
fn def_tts_rate() -> u32 {
    22050
}

fn def_stt_provider() -> String {
    "soniox".into()
}
fn def_stt_model() -> String {
    "stt-rt-v5".into()
}
fn def_language_hints() -> String {
    "ru".into()
}
fn def_utterance_idle_ms() -> u64 {
    2500
}
fn def_max_utterance_chars() -> usize {
    600
}
fn default_temperature() -> f32 {
    0.4
}
fn default_max_tokens() -> u32 {
    700
}
fn default_silence_ms() -> u64 {
    600
}
fn default_max_segment_ms() -> u64 {
    7000
}
fn def_source() -> String {
    "system+mic".into()
}
fn def_mode() -> String {
    "manual".into()
}
fn hk_manual() -> String {
    "Ctrl+2".into()
}
fn hk_hide() -> String {
    "Ctrl+B".into()
}
fn hk_click() -> String {
    "Ctrl+W".into()
}
fn hk_mute() -> String {
    "Ctrl+M".into()
}
fn hk_record() -> String {
    "Ctrl+R".into()
}
fn hk_shot() -> String {
    "Ctrl+H".into()
}
fn hk_shotw() -> String {
    "Ctrl+Shift+H".into()
}
fn hk_tts() -> String {
    "Ctrl+T".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSection {
    #[serde(default = "def_accent")]
    pub accent: String,
    #[serde(default = "def_opacity")]
    pub opacity: u8,
    #[serde(default = "def_indicator_corner")]
    pub indicator_corner: String,
    #[serde(default = "def_protection")]
    pub protection: bool,
    #[serde(default = "def_rail")]
    pub rail: bool,
}

impl Default for UiSection {
    fn default() -> Self {
        Self {
            accent: def_accent(),
            opacity: def_opacity(),
            indicator_corner: def_indicator_corner(),
            protection: def_protection(),
            rail: def_rail(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSection {
    #[serde(default)]
    pub no_focus: bool,
    #[serde(default = "def_move_step")]
    pub move_step: u32,
    #[serde(default = "def_resize_step")]
    pub resize_step: u32,
}

impl Default for WindowSection {
    fn default() -> Self {
        Self {
            no_focus: false,
            move_step: def_move_step(),
            resize_step: def_resize_step(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSection {
    #[serde(default = "def_chat_order")]
    pub order: String,
    #[serde(default = "def_font_size")]
    pub font_size: f32,
    #[serde(default = "def_code_theme")]
    pub code_theme: String,
    #[serde(default = "def_code_scroll")]
    pub code_scroll: bool,
    #[serde(default = "def_autoscroll")]
    pub autoscroll: bool,
    #[serde(default = "def_autoscroll_speed")]
    pub autoscroll_speed: u8,
    #[serde(default = "def_collapse_transcripts")]
    pub collapse_transcripts: bool,
    #[serde(default = "def_collapse_operations")]
    pub collapse_operations: bool,
    #[serde(default = "def_collapse_last")]
    pub collapse_last: bool,
    #[serde(default = "def_compact_quick")]
    pub compact_quick: bool,
    #[serde(default = "def_cancel_on_resend")]
    pub cancel_on_resend: bool,
    #[serde(default = "def_cancel_mode")]
    pub cancel_mode: String,
}

impl Default for ChatSection {
    fn default() -> Self {
        Self {
            order: def_chat_order(),
            font_size: def_font_size(),
            code_theme: def_code_theme(),
            code_scroll: def_code_scroll(),
            autoscroll: def_autoscroll(),
            autoscroll_speed: def_autoscroll_speed(),
            collapse_transcripts: def_collapse_transcripts(),
            collapse_operations: def_collapse_operations(),
            collapse_last: def_collapse_last(),
            compact_quick: def_compact_quick(),
            cancel_on_resend: def_cancel_on_resend(),
            cancel_mode: def_cancel_mode(),
        }
    }
}

fn def_accent() -> String {
    "#f97316".into()
}
fn def_opacity() -> u8 {
    92
}
fn def_indicator_corner() -> String {
    "top-right".into()
}
fn def_protection() -> bool {
    false
}
fn def_rail() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSection {
    #[serde(default = "def_recent_window")]
    pub recent_window: usize,
    #[serde(default = "def_key_turns_cap")]
    pub key_turns_cap: usize,
    #[serde(default = "def_summary_max_tokens")]
    pub summary_max_tokens: u32,
    #[serde(default)]
    pub summary_model: String,
}

impl Default for ContextSection {
    fn default() -> Self {
        Self {
            recent_window: def_recent_window(),
            key_turns_cap: def_key_turns_cap(),
            summary_max_tokens: def_summary_max_tokens(),
            summary_model: String::new(),
        }
    }
}

fn def_recent_window() -> usize {
    12
}
fn def_key_turns_cap() -> usize {
    12
}
fn def_summary_max_tokens() -> u32 {
    300
}
fn def_move_step() -> u32 {
    50
}
fn def_resize_step() -> u32 {
    50
}
fn def_stt_language() -> String {
    "auto".into()
}
fn def_chat_order() -> String {
    "bottom".into()
}
fn def_font_size() -> f32 {
    13.5
}
fn def_code_theme() -> String {
    "github-dark".into()
}
fn def_code_scroll() -> bool {
    true
}
fn def_autoscroll() -> bool {
    true
}
fn def_autoscroll_speed() -> u8 {
    100
}
fn def_collapse_transcripts() -> bool {
    true
}
fn def_collapse_operations() -> bool {
    true
}
fn def_collapse_last() -> bool {
    false
}
fn def_compact_quick() -> bool {
    true
}
fn def_cancel_on_resend() -> bool {
    true
}
fn def_cancel_mode() -> String {
    "drop".into()
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, crate::ConfigError> {
        let mut config = Self::parse(path)?;
        config.normalize();
        config.validate()?;
        Ok(config)
    }

    /// Небомящая миграция к реестру провайдеров:
    /// 1) легаси llm.base_url+api_key без [providers] → [providers.<host>]
    ///    с ТЕМИ ЖЕ credentials (замена на openrouter.ai сломала бы авторизацию —
    ///    STOP Protocol change 030);
    /// 2) пустой provider при непустом реестре → первый по алфавиту;
    /// 3) полностью пусто → дефолт openrouter с записью в реестре.
    pub fn normalize(&mut self) {
        if self.providers.is_empty() && !self.llm.api_key.is_empty() {
            let name = self
                .llm
                .base_url
                .as_deref()
                .map(provider_name_from_host)
                .unwrap_or_else(|| "custom".into());
            self.providers.insert(
                name.clone(),
                ProviderConfig {
                    api_key: self.llm.api_key.clone(),
                    base_url: self.llm.base_url.clone().unwrap_or_default(),
                    enabled: true,
                },
            );
            self.llm.provider = name;
        }
        if self.llm.provider.is_empty() {
            self.llm.provider = if let Some(first) = self.providers.keys().next() {
                first.clone()
            } else {
                "openrouter".into()
            };
        }
        // Запись по умолчанию — только для встроенного openrouter
        // (неизвестные имена НЕ синтезируются: их отсутствие — ошибка).
        if self.llm.provider == "openrouter" {
            self.providers
                .entry("openrouter".into())
                .or_default();
        }
    }

    /// Каталог моделей активного провайдера.
    pub fn get_provider(&self) -> Result<engine_models::OpenAiCompatCatalog, crate::ConfigError> {
        match self.providers.get(&self.llm.provider) {
            Some(p) if p.enabled => Ok(engine_models::OpenAiCompatCatalog::new(
                p.base_url.trim_end_matches('/').to_string(),
                p.api_key.clone(),
            )),
            Some(_) => Err(crate::ConfigError::Validation(format!(
                "provider {} is disabled",
                self.llm.provider
            ))),
            None => Err(crate::ConfigError::Validation(format!(
                "provider {} not found",
                self.llm.provider
            ))),
        }
    }

    /// Parses without validation (used by the tolerant startup fallback).
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self, crate::ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn validate(&self) -> Result<(), crate::ConfigError> {
        if self.vad.silence_ms == 0 {
            return Err(crate::ConfigError::Validation(
                "silence_ms must be > 0".into(),
            ));
        }
        if !(0.0..=2.0).contains(&self.llm.temperature) {
            return Err(crate::ConfigError::Validation(
                "temperature must be 0.0..=2.0".into(),
            ));
        }
        if !matches!(self.audio.source.as_str(), "system+mic" | "system" | "mic") {
            return Err(crate::ConfigError::Validation(
                "audio.source must be system+mic|system|mic".into(),
            ));
        }
        if !matches!(self.audio.mode.as_str(), "vad" | "manual") {
            return Err(crate::ConfigError::Validation(
                "audio.mode must be vad|manual".into(),
            ));
        }
        if !matches!(self.tts.mode.as_str(), "off" | "auto" | "hotkey") {
            return Err(crate::ConfigError::Validation(
                "tts.mode must be off|auto|hotkey".into(),
            ));
        }
        if !(8000..=44100).contains(&self.tts.sample_rate) {
            return Err(crate::ConfigError::Validation(
                "tts.sample_rate must be 8000..=44100".into(),
            ));
        }
        if self.tts.mode != "off" && self.tts.api_key.is_empty() {
            return Err(crate::ConfigError::Validation(
                "tts.api_key is required when tts.mode != off".into(),
            ));
        }
        if !(10..=100).contains(&self.ui.opacity) {
            return Err(crate::ConfigError::Validation(
                "ui.opacity must be 10..=100".into(),
            ));
        }
        if !matches!(
            self.ui.indicator_corner.as_str(),
            "top-right" | "top-left" | "bottom-right" | "bottom-left"
        ) {
            return Err(crate::ConfigError::Validation(
                "ui.indicator_corner must be top-right|top-left|bottom-right|bottom-left".into(),
            ));
        }
        if !matches!(self.stt.language.as_str(), "auto" | "ru" | "en") {
            return Err(crate::ConfigError::Validation(
                "stt.language must be auto|ru|en".into(),
            ));
        }
        if !matches!(self.chat.order.as_str(), "bottom" | "top") {
            return Err(crate::ConfigError::Validation(
                "chat.order must be bottom|top".into(),
            ));
        }
        if !matches!(self.chat.cancel_mode.as_str(), "drop" | "keep") {
            return Err(crate::ConfigError::Validation(
                "chat.cancel_mode must be drop|keep".into(),
            ));
        }
        if self.context.recent_window == 0 {
            return Err(crate::ConfigError::Validation(
                "context.recent_window must be > 0".into(),
            ));
        }
        if self.context.key_turns_cap == 0 {
            return Err(crate::ConfigError::Validation(
                "context.key_turns_cap must be > 0".into(),
            ));
        }
        if self.context.summary_max_tokens == 0 {
            return Err(crate::ConfigError::Validation(
                "context.summary_max_tokens must be > 0".into(),
            ));
        }
        if !self.providers.is_empty() && !self.providers.contains_key(&self.llm.provider) {
            return Err(crate::ConfigError::Validation(format!(
                "llm.provider {} not found in [providers]",
                self.llm.provider
            )));
        }
        Ok(())
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), crate::ConfigError> {
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_models::ModelProvider;

    #[test]
    fn tts_defaults() {
        let c = Config::default();
        assert_eq!(c.tts.mode, "off");
        assert_eq!(c.tts.provider, "cartesia");
        assert_eq!(c.tts.model_id, "sonic-3.5");
        assert_eq!(c.tts.sample_rate, 22050);
        assert!(c.tts.api_key.is_empty());
        assert_eq!(c.hotkeys.tts, "Ctrl+T");
    }

    #[test]
    fn tts_requires_key() {
        let mut c = Config::default();
        c.tts.mode = "auto".into();
        let toml_str = toml::to_string_pretty(&c).unwrap();
        let err = toml::from_str::<Config>(&toml_str)
            .unwrap()
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("api_key"));
    }

    #[test]
    fn tts_validates_mode() {
        let mut c = Config::default();
        c.tts.mode = "nope".into();
        let toml_str = toml::to_string_pretty(&c).unwrap();
        let err = toml::from_str::<Config>(&toml_str)
            .unwrap()
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("tts.mode"));
    }

    #[test]
    fn tts_roundtrip() {
        let mut c = Config::default();
        c.tts.mode = "hotkey".into();
        c.tts.api_key = "sk_test_123".into();
        c.tts.model_id = "sonic-2".into();
        let toml_str = toml::to_string_pretty(&c).unwrap();
        let back: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.tts.mode, "hotkey");
        assert_eq!(back.tts.api_key, "sk_test_123");
        assert_eq!(back.tts.model_id, "sonic-2");
    }

    #[test]
    fn ui_defaults() {
        let c = Config::default();
        assert_eq!(c.ui.accent, "#f97316");
        assert_eq!(c.ui.opacity, 92);
        assert_eq!(c.ui.indicator_corner, "top-right");
        assert!(!c.ui.protection);
        assert!(c.ui.rail);
        assert_eq!(c.context.recent_window, 12);
        assert_eq!(c.context.key_turns_cap, 12);
        assert_eq!(c.context.summary_max_tokens, 300);
        assert!(c.context.summary_model.is_empty());
        assert!(!c.window.no_focus);
        assert_eq!(c.window.move_step, 50);
        assert_eq!(c.chat.order, "bottom");
        assert_eq!(c.chat.font_size, 13.5);
        assert_eq!(c.chat.code_theme, "github-dark");
        assert!(c.chat.collapse_transcripts);
        assert_eq!(c.chat.cancel_mode, "drop");
        assert_eq!(c.stt.language, "auto");
        assert!(!c.llm.search_enabled);
        assert!(c.llm.search_tool_json.is_empty());
    }

    #[test]
    fn validates_opacity() {
        let mut c = Config::default();
        c.ui.opacity = 150;
        let toml_str = toml::to_string_pretty(&c).unwrap();
        let err = toml::from_str::<Config>(&toml_str)
            .unwrap()
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("opacity"));
    }

    #[test]
    fn context_validates() {
        let mut c = Config::default();
        c.context.recent_window = 0;
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("recent_window"));

        let mut c = Config::default();
        c.context.key_turns_cap = 0;
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("key_turns_cap"));

        let mut c = Config::default();
        c.context.summary_max_tokens = 0;
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("summary_max_tokens"));
    }

    #[test]
    fn validates_corner() {
        let mut c = Config::default();
        c.ui.indicator_corner = "center".into();
        let toml_str = toml::to_string_pretty(&c).unwrap();
        let err = toml::from_str::<Config>(&toml_str)
            .unwrap()
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("indicator_corner"));
    }

    #[test]
    fn legacy_migration_nonbreaking() {
        let mut c = Config {
            llm: LlmConfig {
                provider: String::new(),
                api_key: "sk-dslab".into(),
                model: "gpt-5.6-luna".into(),
                base_url: Some("https://api.dslab.tech/v1".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        c.normalize();
        assert_eq!(c.llm.provider, "dslab");
        let p = c.providers.get("dslab").expect("entry synthesized");
        // Credentials НЕ подменяются (STOP Protocol 030).
        assert_eq!(p.api_key, "sk-dslab");
        assert_eq!(p.base_url, "https://api.dslab.tech/v1");
        let cat = c.get_provider().unwrap();
        assert!(cat.base_url().starts_with("https://api.dslab.tech/v1"));
        assert_eq!(cat.api_key(), "sk-dslab");
    }

    #[test]
    fn openrouter_defaults() {
        let mut c = Config::default();
        c.normalize();
        assert_eq!(c.llm.provider, "openrouter");
        let p = c.providers.get("openrouter").expect("default entry");
        assert_eq!(p.base_url, "https://openrouter.ai/api/v1");
        assert!(p.enabled);
    }

    #[test]
    fn validates_provider_exists() {
        let mut c = Config::default();
        c.llm.provider = "unknown".into();
        c.providers.insert(
            "openrouter".into(),
            ProviderConfig {
                api_key: "k".into(),
                ..Default::default()
            },
        );
        let err = c.validate().unwrap_err();
        assert!(err.to_string().contains("unknown"));

        // Нет записи в реестре → get_provider Err, не паника.
        assert!(c.get_provider().is_err());
    }

    #[test]
    fn legacy_toml_roundtrip_keeps_provider() {
        let toml_str = r#"
[llm]
provider = ""
api_key = "sk-x"
model = "gpt-5.6-luna"
base_url = "https://api.dslab.tech/v1"
"#;
        let mut c: Config = toml::from_str(toml_str).unwrap();
        c.normalize();
        assert_eq!(c.llm.provider, "dslab");
        // Сериализация сохраняет реестр.
        let out = toml::to_string_pretty(&c).unwrap();
        assert!(out.contains("[providers.dslab]"));
    }

    #[test]
    fn active_provider_pick() {
        let toml_str = r#"
[llm]
provider = ""
api_key = ""
model = ""
[providers.openrouter]
api_key = "sk-or"
base_url = "https://openrouter.ai/api/v1"
"#;
        let mut c: Config = toml::from_str(toml_str).unwrap();
        c.normalize();
        assert_eq!(c.llm.provider, "openrouter");
        assert_eq!(c.get_provider().unwrap().api_key(), "sk-or");
    }
}
