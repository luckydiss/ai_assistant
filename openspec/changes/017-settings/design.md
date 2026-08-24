# Design: Settings

## 1. Config: новые секции (engine-config/src/config.rs)

Добавить `Serialize` ко ВСЕМ config-struct (для записи файла). В Config добавить:

```rust
#[serde(default)]
pub audio: AudioSection,
#[serde(default)]
pub hotkeys: HotkeysSection,

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AudioSection {
    #[serde(default = "def_source")] pub source: String,   // system+mic | system | mic
    #[serde(default = "def_mode")]   pub mode: String,     // vad | manual
    #[serde(default)] pub mic_device: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotkeysSection {
    #[serde(default = "hk_manual")] pub manual: String,
    #[serde(default = "hk_hide")]   pub hide: String,
    #[serde(default = "hk_click")]  pub click_through: String,
    #[serde(default = "hk_mute")]   pub mute: String,
    #[serde(default = "hk_record")] pub record: String,
    #[serde(default = "hk_shot")]   pub screenshot_full: String,
    #[serde(default = "hk_shotw")]  pub screenshot_region: String,
}
impl Default for AudioSection { ... } impl Default for HotkeysSection { ... }
// дефолты: manual="Ctrl+Shift+Space", hide="Ctrl+B", click_through="Ctrl+W",
// mute="Ctrl+M", record="Ctrl+R", screenshot_full="Ctrl+H", screenshot_region="Ctrl+Shift+H"
```

Новый метод: `pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError>` — `std::fs::write(path, toml::to_string_pretty(self)?)`. Добавить variant `ConfigError::Serialize(#[from] toml::ser::Error)`.

## 2. Gate-функция (engine-orchestrator)

```rust
/// Разрешён ли чанк данного lane в обработку.
pub fn gate(mode: &str, recording: bool, source: &str, is_mic: bool) -> bool {
    let lane_ok = match source {
        "system" => !is_mic,
        "mic" => is_mic,
        _ => true,
    };
    if !lane_ok { return false; }
    match mode { "manual" => recording, _ => true }
}
```

## 3. Hotkey-менеджер (apps/desktop/src/hotkeys.rs)

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};

pub struct HotkeyRegistry { pub map: Mutex<HashMap<String, String>> } // accelerator_str -> action

pub fn register_all(app: &AppHandle, hk: &engine_config::HotkeysSection, reg: &HotkeyRegistry) -> anyhow::Result<()> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let mut map = reg.map.lock().unwrap();
    map.clear();
    let pairs: Vec<(&str, &str)> = vec![
        ("manual", &hk.manual), ("hide", &hk.hide), ("click_through", &hk.click_through),
        ("mute", &hk.mute), ("record", &hk.record),
        ("screenshot_full", &hk.screenshot_full), ("screenshot_region", &hk.screenshot_region),
    ];
    for (action, accel) in pairs {
        if accel.trim().is_empty() { continue; }
        let sc: Shortcut = accel.parse()?;
        gs.register(sc)?;
        map.insert(accel.to_string(), action.to_string());
    }
    Ok(())
}

pub fn action_for(reg: &HotkeyRegistry, shortcut: &Shortcut) -> Option<String> {
    reg.map.lock().unwrap().get(&shortcut.to_string()).cloned()
}
```

В main.rs: плагин строить с `Builder::new().with_handler(|app, sc, ev| { if ev.state == ShortcutState::Pressed { if let Some(a) = action_for(reg, sc) { dispatch(app, &a); } } })`. dispatch: match по action → manual_trigger / hide / click_through / mute / record-toggle / screenshot_full / screenshot_region (018).

Подписка: на ConfigEvent::Changed → `register_all(...)` заново (config берётся из RwLock).

## 4. Команды (commands.rs)

```rust
#[tauri::command] async fn hotkeys_get(cfg: State<'_, Arc<RwLock<Config>>>) -> HotkeysSection
#[tauri::command] async fn set_hotkey(cfg, path: String, action: String, accel: String) -> Result<(), String>
   // валидация: accel.empty ИЛИ accel.parse::<Shortcut>(); затем cfg.write().hotkeys.<action>=accel; cfg.save("config.toml")
#[tauri::command] async fn update_audio_settings(cfg, source: String, mode: String, mic_device: Option<String>) -> Result<(), String>
   // валидация source/mode по множеству; save
#[tauri::command] async fn list_audio_devices() -> Vec<String>
   // cpal::default_host().input_devices() -> names (device.name())
```

set_hotkey принимает action, а не путь — валидация имени action по whitelist.

## 5. Wiring gate в pipeline.rs

В таске audio→segmenters перед process_chunk:
`if !gate(&cfg.audio.mode, recording.load(), &cfg.audio.source, is_mic) { continue; }`
`recording: Arc<AtomicBool>` в AppServices; action "record" инвертирует.
mic_device: AudioEngine::start_mic_capture(device_name: Option<&str>) — lookup через `host.input_devices().find(|d| d.name().ok().as_deref() == Some(name))`, fallback default.

## 6. Settings-view (app.js)

Hash `#settings`: рендер из `hotkeys_get()` — строка «действие + input + Сохранить + статус (Отключено если "")»; секция Запись: select source (3 опции), select mode (2), select mic из `list_audio_devices()`, number chunk (vad.max_segment_ms/1000) → update через update_audio_settings и set_config-команду для vad (добавить `#[tauri::command] update_vad_settings(chunk_ms)` аналогично).

## Рассмотрено и отклонено
- **Захват «нажмите клавишу» в webview:** отклонено — ввод строкой акселератора проще и надёжнее
- **Выбор output-устройства loopback:** отклонено (MVP)
