#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::hotkeys::{action_for, register_all, HotkeyRegistry};
use crate::pipeline::AppServices;
use engine_audio::AudioEngine;
use engine_config::{ConfigEvent, ConfigWatcher};
use engine_context::ContextBuilder;
use engine_dialogue::Speaker;
use engine_llm::LlmClient;
use engine_orchestrator::Orchestrator;
use engine_store::SessionStore;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use stealth::apply_affinity;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::ShortcutState;
use tokio::sync::Mutex;

mod capture;
mod commands;
mod hotkeys;
mod pipeline;
mod stealth;
mod tts;

fn dispatch(app: &AppHandle, action: &str) {
    let services = app.state::<Arc<AppServices>>();
    match action {
        "manual" => {
            tracing::info!("hotkey manual trigger");
            services.orch.manual(None, None);
        }
        "hide" => {
            tracing::info!("hotkey hide overlay");
            if let Some(win) = app.get_webview_window("overlay") {
                let _ = win.hide();
            }
        }
        "click_through" => {
            tracing::info!("hotkey click-through toggle");
            static CLICK_THROUGH: AtomicBool = AtomicBool::new(false);
            let _ = CLICK_THROUGH.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| Some(!v));
            let on = CLICK_THROUGH.load(Ordering::SeqCst);
            if let Some(win) = app.get_webview_window("overlay") {
                let _ = win.set_ignore_cursor_events(on);
            }
        }
        "mute" => {
            tracing::info!("hotkey mute");
            let services_owned = app.state::<Arc<AppServices>>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let audio = services_owned.audio.lock().await;
                audio.set_mic_muted(!audio.mic_muted());
            });
        }
        "record" => {
            tracing::info!("hotkey record toggle");
            let _ = services.recording.fetch_xor(true, Ordering::SeqCst);
            let rec = services.recording.load(Ordering::SeqCst);
            let _ = app.emit("status", if rec { "recording" } else { "paused" });
        }
        "screenshot_full" | "screenshot_region" => {
            tracing::info!("hotkey {action} (018 screenshots)");
            let region = action == "screenshot_region";
            let services_owned = app.state::<Arc<AppServices>>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let _ = commands::screen_analyze_inner(&services_owned, region).await;
            });
        }
        "tts" => {
            tracing::info!("hotkey tts");
            let services_owned = app.state::<Arc<AppServices>>().inner().clone();
            let cfg_state = app.state::<Arc<std::sync::RwLock<engine_config::Config>>>();
            let cfg = cfg_state.read().map(|g| g.clone()).unwrap_or_default();
            tauri::async_runtime::spawn(async move {
                let _ = crate::tts::play_last(&services_owned, &cfg).await;
            });
        }
        other => tracing::warn!("unknown hotkey action: {other}"),
    }
}

fn resolve_base_dir() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    if cwd.join("config.toml").is_file() {
        return Some(cwd);
    }
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    if dir.join("config.toml").is_file() {
        return Some(dir.to_path_buf());
    }
    let root = dir.parent()?.parent()?.to_path_buf();
    if root.join("config.toml").is_file() {
        return Some(root);
    }
    None
}

fn main() -> anyhow::Result<()> {
    if let Some(base) = resolve_base_dir() {
        let _ = std::env::set_current_dir(base);
    }
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let rt = Box::leak(Box::new(tokio::runtime::Runtime::new()?));
    let registry = Arc::new(HotkeyRegistry::new());
    let reg = registry.clone();
    let handler = move |app: &AppHandle,
                        sc: &tauri_plugin_global_shortcut::Shortcut,
                        ev: tauri_plugin_global_shortcut::ShortcutEvent| {
        if ev.state == ShortcutState::Pressed {
            if let Some(action) = action_for(&reg, sc) {
                dispatch(app, &action);
            }
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(handler).build())
        .setup(move |app| {
            let _guard = rt.enter();
            let cfg_path = "config.toml";
            let watcher = ConfigWatcher::start(cfg_path)?;
            let cfg = match watcher.config().read() {
                Ok(g) => g.clone(),
                Err(_) => return Err("config lock poisoned".into()),
            };

            // --- components ---
            let audio = Arc::new(Mutex::new(AudioEngine::new()));
            app.manage(watcher.config().clone());
            let ctx = ContextBuilder::new(
                cfg.prompts.system.clone(),
                cfg.prompts.persona.clone(),
                8000,
            );
            let llm = LlmClient::new(
                cfg.llm
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "https://api.openai.com/v1".into()),
                cfg.llm.api_key.clone(),
                cfg.llm.model.clone(),
                cfg.llm.temperature,
                cfg.llm.max_tokens,
                cfg.llm.reasoning_effort.clone(),
            )?
            .with_search(cfg.llm.search_enabled, cfg.llm.search_tool_json.clone());
            let orch = Arc::new(Orchestrator::new(ctx, llm, false));

            // --- session store + app services ---
            let store = Arc::new(Mutex::new(SessionStore::open("history.db")?));
            app.manage(store.clone());
            let persist_store = store.clone();
            orch.set_persist(Some(Arc::new(move |chat_id, turns| {
                let store = persist_store.clone();
                tauri::async_runtime::spawn(async move {
                    let msgs: Vec<engine_store::ChatMsg> = turns
                        .iter()
                        .map(|t| {
                            let speaker = match t.speaker {
                                Speaker::Interviewer => "I",
                                Speaker::Candidate => "C",
                            };
                            engine_store::ChatMsg {
                                speaker: speaker.into(),
                                text: t.text.clone(),
                                at: t.start_time.to_rfc3339(),
                            }
                        })
                        .collect();
                    let g = store.lock().await;
                    let _ = g.save_chat_msgs(&chat_id, &msgs);
                });
            })));
            let services = Arc::new(AppServices {
                store: store.clone(),
                audio: audio.clone(),
                orch: orch.clone(),
                pipeline: Mutex::new(None),
                recording: Arc::new(AtomicBool::new(false)),
                tts: Mutex::new(crate::tts::TtsState::new()),
                active_meeting: Arc::new(tokio::sync::Mutex::new(None)),
                auto: Arc::new(AtomicBool::new(false)),
                notes_rag: Arc::new(AtomicBool::new(false)),
                rail_visible: Arc::new(AtomicBool::new(true)),
            });
            app.manage(services.clone());

            // --- hotkeys (register from config, re-register on change) ---
            let reg_app = registry.clone();
            register_all(app.handle(), &cfg.hotkeys, &reg_app)?;
            let watcher_cfg = watcher.config().clone();
            let reg_watcher = registry.clone();
            let app_handle = app.handle().clone();
            let mut rx = watcher.events();
            tokio::spawn(async move {
                while let Ok(ev) = rx.recv().await {
                    if matches!(ev, ConfigEvent::Changed) {
                        let cfg = watcher_cfg.read().map(|g| g.clone());
                        if let Ok(cfg) = cfg {
                            let _ = register_all(&app_handle, &cfg.hotkeys, &reg_watcher);
                        }
                    }
                }
                Ok::<(), ()>(())
            });

            // --- overlay window ---
            let overlay = WebviewWindowBuilder::new(
                app,
                "overlay",
                WebviewUrl::App("overlay.html".into()),
            )
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .resizable(true)
            .inner_size(480.0, 640.0)
            .position(900.0, 60.0)
            .build()?;
            if cfg.ui.protection {
                apply_affinity(&overlay)?;
            }
            overlay.hide()?;

            // --- main window stealth ---
            if cfg.ui.protection {
                apply_affinity(&app.get_webview_window("main").ok_or_else(|| {
                    anyhow::anyhow!("main window not found")
                })?)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::manual_trigger,
            commands::screen_analyze,
            commands::meetings_list,
            commands::meeting_create,
            commands::meeting_rename,
            commands::meeting_delete,
            commands::contexts_list,
            commands::context_save,
            commands::context_delete,
            commands::meeting_set_context,
            commands::start_pipeline,
            commands::stop_pipeline,
            commands::mic_mute,
            commands::protection_status,
            commands::protection_set,
            commands::click_through,
            commands::hotkeys_get,
            commands::set_hotkey,
            commands::update_audio_settings,
            commands::list_audio_devices,
            commands::get_config,
            commands::tts_play_last,
            commands::tts_speak,
            commands::tts_set_mode,
            commands::tts_auto_set,
            commands::tts_auto_get,
            commands::ui_set,
            commands::chats_list,
            commands::chat_create,
            commands::chat_switch,
            commands::chat_messages,
            commands::notes_list,
            commands::note_get,
            commands::stt_get,
            commands::stt_set,
            commands::ui_get,
            commands::go_home,
            commands::auto_answers_set,
            commands::auto_answers_get,
            commands::search_set,
            commands::notes_rag_set,
            commands::ctx_reset,
            commands::indicator_get
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
