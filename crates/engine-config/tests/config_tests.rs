use engine_config::{Config, ConfigError, ConfigEvent, ConfigWatcher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_path() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("engine-config-{}-{}.toml", std::process::id(), id))
}

fn valid_config() -> String {
    r#"
[stt]
provider = "groq"
api_key = "gsk_test"
model = "whisper-large-v3-turbo"
chunk_ms = 7000

[llm]
provider = "openai"
api_key = "sk_test"
model = "gpt-4o-mini"
base_url = "https://api.openai.com/v1"

[vad]
silence_ms = 600
max_segment_ms = 7000

[prompts]
system = "prompt"
persona = "persona"
"#
    .to_string()
}

#[test]
fn loads_valid_config() {
    let path = temp_path();
    std::fs::write(&path, valid_config()).unwrap();

    let config = Config::load(&path).unwrap();
    assert_eq!(config.stt.provider, "groq");
    assert_eq!(config.stt.soniox_api_key, "");
    assert_eq!(config.stt.language_hints, "ru");
    assert_eq!(config.llm.model, "gpt-4o-mini");
    assert_eq!(config.vad.silence_ms, 600);
    assert_eq!(config.prompts.system, "prompt");
}

#[test]
fn errors_on_missing_file() {
    let path = temp_path();

    let err = Config::load(&path).unwrap_err();
    assert!(matches!(err, ConfigError::Io(_)));
}

#[test]
fn errors_on_invalid_toml() {
    let path = temp_path();
    std::fs::write(&path, "this is [ not valid toml").unwrap();

    let err = Config::load(&path).unwrap_err();
    assert!(matches!(err, ConfigError::Parse(_)));
}

#[test]
fn applies_defaults() {
    let path = temp_path();
    std::fs::write(
        &path,
        r#"
[stt]
provider = "groq"
api_key = "gsk_test"
model = "whisper-large-v3-turbo"
"#,
    )
    .unwrap();

    let config = Config::load(&path).unwrap();
    assert_eq!(config.llm.temperature, 0.4);
    assert_eq!(config.llm.max_tokens, 700);
    assert_eq!(config.llm.reasoning_effort, None);
    assert_eq!(config.vad.silence_ms, 600);
    assert_eq!(config.vad.max_segment_ms, 7000);
}

#[test]
fn validates_thresholds() {
    let path = temp_path();
    std::fs::write(
        &path,
        r#"
[stt]
provider = "groq"
api_key = "gsk_test"
model = "whisper-large-v3-turbo"

[vad]
silence_ms = 0
max_segment_ms = 7000

[llm]
provider = "openai"
api_key = "sk_test"
model = "gpt-4o-mini"

[prompts]
system = "prompt"
persona = "persona"
"#,
    )
    .unwrap();

    let err = Config::load(&path).unwrap_err();
    assert!(matches!(err, ConfigError::Validation(_)));
}

#[tokio::test]
async fn reloads_on_change() {
    let path = temp_path();
    std::fs::write(&path, valid_config()).unwrap();

    let watcher = ConfigWatcher::start(&path).unwrap();
    let config = watcher.config().clone();
    let mut events = watcher.events();
    let original_model = config.read().unwrap().stt.model.clone();

    let new_config = valid_config().replace(
        "model = \"whisper-large-v3-turbo\"",
        "model = \"whisper-small\"",
    );
    std::fs::write(&path, new_config).unwrap();

    let event = tokio::time::timeout(Duration::from_secs(5), events.recv())
        .await
        .expect("timeout waiting for change event")
        .unwrap();
    assert!(matches!(event, ConfigEvent::Changed));

    assert_ne!(original_model, "whisper-small");
    assert_eq!(config.read().unwrap().stt.model, "whisper-small");
}

#[tokio::test]
async fn keeps_old_on_error() {
    let path = temp_path();
    std::fs::write(&path, valid_config()).unwrap();

    let watcher = ConfigWatcher::start(&path).unwrap();
    let config = watcher.config().clone();
    let mut events = watcher.events();
    let original = config.read().unwrap().stt.model.clone();

    std::fs::write(&path, "this is [ not valid toml").unwrap();

    let event = tokio::time::timeout(Duration::from_secs(5), events.recv())
        .await
        .expect("timeout waiting for error event")
        .unwrap();
    assert!(matches!(event, ConfigEvent::Error(_)));

    assert_eq!(config.read().unwrap().stt.model, original);
}

#[test]
fn stt_soniox_defaults() {
    let c = Config::default();
    assert_eq!(c.stt.provider, "soniox");
    assert_eq!(c.stt.model, "stt-rt-v5");
    assert_eq!(c.stt.language_hints, "ru");
    assert_eq!(c.stt.soniox_api_key, "");
    assert_eq!(c.stt.utterance_idle_ms, 2500);
    assert_eq!(c.stt.max_utterance_chars, 600);
}

#[test]
fn chunk_ms_removed() {
    let path = temp_path();
    std::fs::write(&path, valid_config()).unwrap();
    let c = Config::load(&path).unwrap();
    let toml_str = toml::to_string_pretty(&c).unwrap();
    assert!(!toml_str.contains("chunk_ms"));
}

#[test]
fn toml_roundtrip_keeps_hotkeys() {
    let path = temp_path();
    std::fs::write(&path, valid_config()).unwrap();

    let mut config = Config::load(&path).unwrap();
    config.hotkeys.manual = "Ctrl+2".into();
    config.audio.mode = "vad".into();
    config.save(&path).unwrap();

    let reloaded = Config::load(&path).unwrap();
    assert_eq!(reloaded.hotkeys.manual, "Ctrl+2");
    assert_eq!(reloaded.hotkeys.hide, "Ctrl+B");
    assert_eq!(reloaded.hotkeys.click_through, "Ctrl+W");
    assert_eq!(reloaded.hotkeys.mute, "Ctrl+M");
    assert_eq!(reloaded.hotkeys.record, "Ctrl+R");
    assert_eq!(reloaded.hotkeys.screenshot_full, "Ctrl+H");
    assert_eq!(reloaded.hotkeys.screenshot_region, "Ctrl+Shift+H");
    assert_eq!(reloaded.audio.mode, "vad");
    assert_eq!(reloaded.audio.source, "system+mic");
}

#[test]
fn validates_audio_section() {
    let path = temp_path();
    let mut cfg = valid_config();
    cfg += "[audio]\nsource = \"bogus\"\n";
    std::fs::write(&path, cfg).unwrap();
    let err = Config::load(&path).unwrap_err();
    assert!(matches!(err, ConfigError::Validation(_)));
}
