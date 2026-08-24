use std::collections::HashMap;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub struct HotkeyRegistry {
    pub map: Mutex<HashMap<String, String>>,
}

impl HotkeyRegistry {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }
}

pub fn register_all(
    app: &AppHandle,
    hk: &engine_config::HotkeysSection,
    reg: &HotkeyRegistry,
) -> anyhow::Result<()> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let mut map = reg.map.lock().unwrap();
    map.clear();
    let pairs: Vec<(&str, &String)> = vec![
        ("manual", &hk.manual),
        ("hide", &hk.hide),
        ("click_through", &hk.click_through),
        ("mute", &hk.mute),
        ("record", &hk.record),
        ("screenshot_full", &hk.screenshot_full),
        ("screenshot_region", &hk.screenshot_region),
        ("tts", &hk.tts),
    ];
    for (action, accel) in pairs {
        if accel.trim().is_empty() {
            continue;
        }
        let sc: Shortcut = match accel.parse() {
            Ok(sc) => sc,
            Err(e) => {
                tracing::warn!("hotkey {action}: bad accel {accel:?}: {e}");
                continue;
            }
        };
        match gs.register(sc) {
            Ok(()) => {
                map.insert(sc.to_string(), action.to_string());
            }
            Err(e) => {
                // e.g. another instance already holds the hotkey — log, don't crash setup.
                tracing::warn!("hotkey {action} ({accel}): registration failed: {e}");
            }
        }
    }
    Ok(())
}

pub fn action_for(reg: &HotkeyRegistry, shortcut: &Shortcut) -> Option<String> {
    reg.map.lock().unwrap().get(&shortcut.to_string()).cloned()
}
