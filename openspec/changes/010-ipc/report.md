# Report: IPC Wiring

## Result

`apps/desktop/src/main.rs` — composition root, связывающий все движки: config → audio → VAD (2 лейна) → STT → dialogue → orchestrator → LLM → UI (emit), плюс хоткеи Ctrl+Shift+Space (manual trigger) и Ctrl+Shift+H (hide/show), capabilities/default.json с global-shortcut. Desktop и workspace собираются, clippy/fmt чисты.

## Deviations from Design

1. **`ConfigWatcher::start` API:** дизайн §3 предполагает `let (cfg_lock, _cfg_events) = ConfigWatcher::start(...)?; cfg_lock.read().unwrap()`, но реализация 002 возвращает `ConfigWatcher` с `.config() -> &Arc<RwLock<Config>>` и `.events()`. Адаптировано: `watcher.config().read()` + guard-проверка (вместо `unwrap`).

2. **`SttProcessor::new(...)?` и `LlmClient::new(...)?`:** из-за `Result`-сигнатур (изменения 005/008) — в setup используется `?` вместо дизайнового прямого вызова.

3. **`#[tauri::command] manual_trigger` возвращает `Result<(), String>`:** Tauri v2 требует, чтобы async-команда со ссылочным `State` возвращала `Result` (иначе ошибка макроса). Дизайн §3 объявлял её без возврата.

4. **`use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState}`** — вынесен в импорты файла; в дизайне `use ShortcutState` стоит внутри setup и без трейта `GlobalShortcutExt` метод `app.global_shortcut()` не виден (E0599).

5. **`anyhow::bail!` в setup не подходит:** setup возвращает `Result<(), Box<dyn Error>>`, поэтому для poisoned-лока — `return Err("...".into())`.

6. **`serde_json` добавлен в deps desktop** для `handle2.emit("turn", serde_json::json!({...}))`.

## Verified

- `cargo metadata --no-deps` — ok
- `cargo build -p desktop` — ok
- `cargo clippy -p desktop -- -D warnings` — ok
- `cargo fmt -p desktop --check` — ok
- `cargo build --workspace` — ok

## Not Verified (manual)

- 3.1 Smoke `cargo run -p desktop` с config.toml + silero_vad.onnx: требует работающих WASAPI-устройств (системный аудио + микрофон) и GUI-окна; на CI-машине не выполнялся.
- 3.2 Хоткей Ctrl+Shift+Space (запрос к LLM требует валидный LLM API key в config.toml).
- 3.3 Хоткей Ctrl+Shift+H (hide/show окна).

Все три — manual-проверки, зависящие от окружения. Отмечены в tasks.md как выполнимые только на машине с аудио-устройствами.