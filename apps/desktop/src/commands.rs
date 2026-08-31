use crate::pipeline::{start as pipeline_start, stop as pipeline_stop, AppServices};
use engine_config::{ChatSection, UiSection};
use engine_dialogue::{Speaker, Turn};
use engine_models::ModelMetadata;
use engine_store::{ChatRow, ContextRow, MeetingRow, NoteRow, SessionStore};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;

type Store = Arc<Mutex<SessionStore>>;
type Services = Arc<AppServices>;

/// Каталог чат-моделей активного провайдера (capabilities-driven фильтрация
/// внутри engine-models, change 030).
#[tauri::command]
pub async fn models_list(cfg: State<'_, ConfigState>) -> Result<Vec<ModelMetadata>, String> {
    let provider = {
        let c = cfg.read().map_err(|e| e.to_string())?;
        c.get_provider().map_err(|e| e.to_string())?
    };
    use engine_models::ModelProvider;
    let all = provider.list_models().await.map_err(|e| e.to_string())?;
    Ok(all.into_iter().filter(|m| m.is_chat()).collect())
}

/// Сменить модель и уровень рассуждений (живо, с сохранением в config.toml).
/// Модель валидируется по каталогу провайдера до переключения живого клиента.
#[tauri::command]
pub async fn llm_set(
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
    model: String,
    effort: Option<String>,
) -> Result<(), String> {
    let provider = {
        let c = cfg.read().map_err(|e| e.to_string())?;
        c.get_provider().map_err(|e| e.to_string())?
    };
    use engine_models::ModelProvider;
    provider.validate_model(&model).await.map_err(|e| e.to_string())?;
    {
        let mut guard = cfg.write().map_err(|e| e.to_string())?;
        guard.llm.model = model.clone();
        guard.llm.reasoning_effort = effort.clone();
        // Реестр провайдеров тоже сохраняем (после нормализации при старте).
        let out = guard.clone();
        drop(guard);
        out.save("config.toml").map_err(|e| e.to_string())?;
    }
    services.orch.set_llm(model, effort);
    Ok(())
}

/// Горячие промпты: пересобрать ContextBuilder из текущего конфига + контекста
/// встречи и отправить в оркестратор. meeting_id=None → активная встреча.
pub(crate) async fn sync_ctx_builder(
    app: &AppHandle,
    services: &Arc<AppServices>,
    meeting_id: Option<String>,
) {
    let Some(cfg) = cfg_of(app) else { return };
    let mid = match meeting_id {
        Some(m) => Some(m),
        None => services.active_meeting.lock().await.clone(),
    };
    let store = services.store.lock().await;
    let meeting = mid.as_deref().and_then(|mid| {
        store
            .list_meetings()
            .ok()
            .and_then(|ms| ms.into_iter().find(|m| m.id == mid))
    });
    let ws = meeting
        .as_ref()
        .and_then(|m| m.context_id.clone())
        .and_then(|cid| store.get_context(&cid).ok())
        .map(|c| engine_context::PromptContext {
            base_system: String::new(),
            role: if c.languages.is_empty() {
                c.role.clone()
            } else {
                format!("{} ({})", c.role, c.languages.join(", "))
            },
            extra_prompt: c.extra_prompt.clone(),
            resume_text: c.resume_text.clone(),
            vacancy: meeting
                .as_ref()
                .map(|m| m.vacancy.clone())
                .unwrap_or_default(),
        });
    let builder = match &ws {
        // Промпт контекста задан → он ЗАМЕНЯЕТ основной system-промпт
        // (и для авто, и для ручных запросов).
        Some(ws) if !ws.extra_prompt.trim().is_empty() => {
            let mut persona = ws.role.clone();
            if !ws.resume_text.is_empty() {
                persona.push_str(&format!("\nРезюме кандидата: {}", ws.resume_text));
            }
            if !ws.vacancy.is_empty() {
                persona.push_str(&format!("\nВакансия: {}", ws.vacancy));
            }
            engine_context::ContextBuilder::new(ws.extra_prompt.clone(), persona, 8000)
                .with_manual_system(ws.extra_prompt.clone())
        }
        Some(ws) => {
            engine_context::ContextBuilder::with_workspace(cfg.prompts.system.clone(), ws, 8000)
                .with_manual_system(cfg.prompts.manual_system.clone())
        }
        None => engine_context::ContextBuilder::new(
            cfg.prompts.system.clone(),
            cfg.prompts.persona.clone(),
            8000,
        )
        .with_manual_system(cfg.prompts.manual_system.clone()),
    };
    {
        let probe = builder.build(&engine_context::ContextInput::new(&[]));
        let sys = match probe.first() {
            Some(m) => match &m.content {
                engine_context::MessageContent::Text(t) => t.clone(),
                _ => String::new(),
            },
            None => String::new(),
        };
        tracing::info!(
            "sync_ctx_builder: mid={:?} role={:?} extra_len={} sys_head={:?}",
            mid,
            ws.as_ref().map(|w| &w.role),
            ws.as_ref().map(|w| w.extra_prompt.len()).unwrap_or(0),
            sys.chars().take(80).collect::<String>()
        );
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("ctx_debug.log")
            .and_then(|mut f| {
                use std::io::Write;
                writeln!(
                    f,
                    "mid={:?} role={:?} extra_len={} sys_head={:?}",
                    mid,
                    ws.as_ref().map(|w| &w.role),
                    ws.as_ref().map(|w| w.extra_prompt.len()).unwrap_or(0),
                    sys.chars().take(120).collect::<String>()
                )
            });
    }
    services.orch.set_ctx(builder);
}

#[tauri::command]
pub async fn manual_trigger(
    app: AppHandle,
    services: State<'_, Services>,
    note: Option<String>,
) -> Result<(), String> {
    sync_ctx_builder(&app, &services, None).await;
    services.orch.manual(note, None);
    Ok(())
}

#[tauri::command]
pub async fn screen_analyze(
    app: AppHandle,
    services: State<'_, Services>,
    window_only: bool,
) -> Result<(), String> {
    screen_analyze_inner(app, &services, window_only).await
}

pub(crate) async fn screen_analyze_inner(
    app: AppHandle,
    services: &Arc<AppServices>,
    window_only: bool,
) -> Result<(), String> {
    use crate::capture::{capture_active_window, capture_virtual_screen, encode_png};
    use base64::Engine;
    tracing::info!("screen_analyze: capturing (window_only={window_only})");
    let rgba = if window_only {
        capture_active_window().map_err(|e| e.to_string())?
    } else {
        capture_virtual_screen().map_err(|e| e.to_string())?
    };
    tracing::info!("screen_analyze: captured {}x{}", rgba.w, rgba.h);
    let png = encode_png(&rgba).map_err(|e| e.to_string())?;
    tracing::info!("screen_analyze: png {} bytes", png.len());
    let b64 = base64::engine::general_purpose::STANDARD.encode(png);
    sync_ctx_builder(&app, services, None).await;
    services.orch.manual(
        Some("Проанализируй скриншот и помоги с задачей на экране".into()),
        Some(b64),
    );
    tracing::info!("screen_analyze: sent to orchestrator");
    Ok(())
}

#[tauri::command]
pub async fn meetings_list(state: State<'_, Store>) -> Result<Vec<MeetingRow>, String> {
    state.lock().await.list_meetings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn meeting_create(
    app: AppHandle,
    services: State<'_, Services>,
    state: State<'_, Store>,
    name: String,
    vacancy: String,
) -> Result<String, String> {
    let id = state
        .lock()
        .await
        .create_meeting(&name, &vacancy)
        .map_err(|e| e.to_string())?;
    // Новая встреча наследует последний использованный контекст.
    let last_ctx = {
        let store = state.lock().await;
        store
            .list_meetings()
            .ok()
            .and_then(|ms| {
                ms.into_iter()
                    .find(|m| m.context_id.as_deref().is_some_and(|c| !c.is_empty()))
            })
            .and_then(|m| m.context_id)
    };
    if let Some(cid) = last_ctx {
        state
            .lock()
            .await
            .set_meeting_context(&id, &cid)
            .map_err(|e| e.to_string())?;
    }
    sync_ctx_builder(&app, &services, Some(id.clone())).await;
    Ok(id)
}

#[tauri::command]
pub async fn meeting_rename(
    state: State<'_, Store>,
    id: String,
    name: String,
) -> Result<(), String> {
    state
        .lock()
        .await
        .rename_meeting(&id, &name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn meeting_delete(state: State<'_, Store>, id: String) -> Result<(), String> {
    state
        .lock()
        .await
        .delete_meeting(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn contexts_list(state: State<'_, Store>) -> Result<Vec<ContextRow>, String> {
    state.lock().await.list_contexts().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn context_save(state: State<'_, Store>, ctx: ContextRow) -> Result<(), String> {
    let store = state.lock().await;
    let result = if store.get_context(&ctx.id).is_ok() {
        store.update_context(&ctx)
    } else {
        store.create_context(&ctx)
    };
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn context_delete(state: State<'_, Store>, id: String) -> Result<(), String> {
    state
        .lock()
        .await
        .delete_context(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn meeting_set_context(
    state: State<'_, Store>,
    meeting_id: String,
    context_id: String,
) -> Result<(), String> {
    state
        .lock()
        .await
        .set_meeting_context(&meeting_id, &context_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_pipeline(
    app: AppHandle,
    services: State<'_, Services>,
    meeting_id: String,
) -> Result<(), String> {
    {
        let store = services.store.lock().await;
        let chats = store.list_chats(&meeting_id).map_err(|e| e.to_string())?;
        let chat_id = if let Some(first) = chats.first() {
            first.id.clone()
        } else {
            store.create_chat(&meeting_id).map_err(|e| e.to_string())?.id
        };
        for ch in &chats {
            let msgs = store.chat_msgs(&ch.id).map_err(|e| e.to_string())?;
            let turns: Vec<Turn> = msgs
                .iter()
                .filter_map(|m| {
                    let (speaker, typed) = match m.speaker.as_str() {
                        "I" => (Speaker::Interviewer, false),
                        "user" => (Speaker::Interviewer, true),
                        _ => (Speaker::Candidate, false),
                    };
                    let at = chrono::DateTime::parse_from_rfc3339(&m.at)
                        .ok()?
                        .with_timezone(&chrono::Utc);
                    Some(Turn {
                        speaker,
                        text: m.text.clone(),
                        start_time: at,
                        end_time: at,
                        typed,
                    })
                })
                .collect();
            services.orch.load_chat(ch.id.clone(), turns);
        }
        services.orch.set_active_chat(chat_id);
    }
    // Горячие промпты + персона встречи (роль/резюме/вакансия перекрывают config.toml).
    sync_ctx_builder(&app, &services, Some(meeting_id.clone())).await;
    let handle = pipeline_start(app.clone(), services.store.clone(), meeting_id.clone())
        .await
        .map_err(|e| e.to_string())?;
    *services.pipeline.lock().await = Some(handle);
    services.recording.store(true, Ordering::SeqCst);
    *services.active_meeting.lock().await = Some(meeting_id.clone());
    emit_indicator(&app, &services, &cfg_of(&app).unwrap_or_default());
    let _ = app.emit("meeting", meeting_id);
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.show();
    }
    Ok(())
}

#[tauri::command]
pub async fn chat_messages(
    state: State<'_, Store>,
    id: String,
) -> Result<Vec<engine_store::ChatMsg>, String> {
    state.lock().await.chat_msgs(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_pipeline(app: AppHandle, services: State<'_, Services>) -> Result<(), String> {
    let mut guard = services.pipeline.lock().await;
    if let Some(h) = guard.as_mut() {
        pipeline_stop(h, &services.audio).await;
    }
    *guard = None;
    services.recording.store(false, Ordering::SeqCst);
    crate::tts::reset(services.inner()).await;
    emit_indicator(&app, &services, &cfg_of(&app).unwrap_or_default());
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.hide();
    }
    Ok(())
}

#[tauri::command]
pub async fn mic_mute(services: State<'_, Services>, muted: bool) -> Result<(), String> {
    services.audio.lock().await.set_mic_muted(muted);
    Ok(())
}

fn protection_on(app: &AppHandle) -> Result<bool, String> {
    use raw_window_handle::HasWindowHandle;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE,
    };
    let Some(win) = app.get_webview_window("overlay") else {
        return Ok(false);
    };
    let raw = win.window_handle().map_err(|e| e.to_string())?.as_raw();
    let hwnd = match raw {
        raw_window_handle::RawWindowHandle::Win32(h) => HWND(h.hwnd.get()),
        _ => return Ok(false),
    };
    let mut current = 0u32;
    unsafe { GetWindowDisplayAffinity(hwnd, &mut current).map_err(|e| e.to_string())? };
    Ok(current == WDA_EXCLUDEFROMCAPTURE.0)
}

#[tauri::command]
pub async fn protection_status(app: AppHandle) -> Result<bool, String> {
    protection_on(&app)
}

#[tauri::command]
pub async fn protection_set(
    app: AppHandle,
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
    on: bool,
) -> Result<(), String> {
    for name in ["overlay", "main"] {
        if let Some(win) = app.get_webview_window(name) {
            let res = if on {
                crate::stealth::apply_affinity(&win)
            } else {
                crate::stealth::clear_affinity(&win)
            };
            res.map_err(|e| e.to_string())?;
        }
    }
    {
        let mut guard = cfg.write().map_err(|e| e.to_string())?;
        guard.ui.protection = on;
        let out = guard.clone();
        drop(guard);
        out.save("config.toml").map_err(|e| e.to_string())?;
    }
    if let Some(cfg_v) = cfg_of(&app) {
        emit_indicator(&app, &services, &cfg_v);
    }
    Ok(())
}

#[derive(Serialize, Clone)]
pub struct IndicatorView {
    pub protection: bool,
    pub recording: bool,
    pub auto: bool,
    pub tts: String,
}

pub(crate) fn indicator_state(
    app: &AppHandle,
    services: &Arc<AppServices>,
    cfg: &engine_config::Config,
) -> IndicatorView {
    IndicatorView {
        protection: protection_on(app).unwrap_or(false),
        recording: services.recording.load(Ordering::SeqCst),
        auto: services.auto.load(Ordering::SeqCst),
        tts: cfg.tts.mode.clone(),
    }
}

pub(crate) fn emit_indicator(
    app: &AppHandle,
    services: &Arc<AppServices>,
    cfg: &engine_config::Config,
) {
    let _ = app.emit("indicator", indicator_state(app, services, cfg));
}

#[tauri::command]
pub async fn indicator_get(
    app: AppHandle,
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
) -> Result<IndicatorView, String> {
    let cfg = cfg.read().map_err(|e| e.to_string())?.clone();
    Ok(indicator_state(&app, &services, &cfg))
}

#[tauri::command]
pub async fn click_through(app: AppHandle, on: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("overlay") {
        win.set_ignore_cursor_events(on).map_err(|e| e.to_string())?;
    }
    Ok(())
}

type ConfigState = Arc<std::sync::RwLock<engine_config::Config>>;

pub(crate) fn cfg_of(app: &AppHandle) -> Option<engine_config::Config> {
    app.try_state::<ConfigState>()?.read().ok().map(|g| g.clone())
}

#[tauri::command]
pub async fn hotkeys_get(cfg: State<'_, ConfigState>) -> Result<engine_config::HotkeysSection, String> {
    Ok(cfg.read().map_err(|e| e.to_string())?.hotkeys.clone())
}

#[tauri::command]
pub async fn get_config(cfg: State<'_, ConfigState>) -> Result<engine_config::Config, String> {
    Ok(cfg.read().map_err(|e| e.to_string())?.clone())
}

#[tauri::command]
pub async fn cancel_generation(
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
) -> Result<(), String> {
    let keep = cfg.read().map_err(|e| e.to_string())?.chat.cancel_mode == "keep";
    services.orch.cancel(keep);
    Ok(())
}

#[tauri::command]
pub async fn config_set(
    app: AppHandle,
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
    section: String,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let out = {
    let mut guard = cfg.write().map_err(|e| e.to_string())?;
        match (section.as_str(), key.as_str()) {
            ("chat", "order") => guard.chat.order = value.as_str().ok_or("expected string")?.into(),
        ("chat", "font_size") => {
            guard.chat.font_size = value.as_f64().ok_or("expected number")? as f32;
        }
        ("chat", "code_theme") => {
            guard.chat.code_theme = value.as_str().ok_or("expected string")?.into();
        }
        ("chat", "code_scroll") => {
            guard.chat.code_scroll = value.as_bool().ok_or("expected bool")?;
        }
        ("chat", "autoscroll") => {
            guard.chat.autoscroll = value.as_bool().ok_or("expected bool")?;
        }
        ("chat", "autoscroll_speed") => {
            guard.chat.autoscroll_speed = value.as_u64().ok_or("expected number")? as u8;
        }
        ("chat", "collapse_transcripts") => {
            guard.chat.collapse_transcripts = value.as_bool().ok_or("expected bool")?;
        }
        ("chat", "collapse_operations") => {
            guard.chat.collapse_operations = value.as_bool().ok_or("expected bool")?;
        }
        ("chat", "collapse_last") => {
            guard.chat.collapse_last = value.as_bool().ok_or("expected bool")?;
        }
        ("chat", "compact_quick") => {
            guard.chat.compact_quick = value.as_bool().ok_or("expected bool")?;
        }
        ("chat", "cancel_on_resend") => {
            guard.chat.cancel_on_resend = value.as_bool().ok_or("expected bool")?;
            services
                .orch
                .set_cancel_policy(guard.chat.cancel_on_resend, guard.chat.cancel_mode == "keep");
        }
        ("chat", "cancel_mode") => {
            let m = value.as_str().ok_or("expected string")?;
            if !matches!(m, "drop" | "keep") {
                return Err(format!("invalid cancel_mode: {m}"));
            }
            guard.chat.cancel_mode = m.into();
            services
                .orch
                .set_cancel_policy(guard.chat.cancel_on_resend, m == "keep");
        }
        ("ui", "accent") => {
            guard.ui.accent = value.as_str().ok_or("expected string")?.into();
        }        ("ui", "opacity") => {
            guard.ui.opacity = value.as_u64().ok_or("expected number")? as u8;
        }
        ("ui", "indicator_corner") => {
            guard.ui.indicator_corner = value.as_str().ok_or("expected string")?.into();
        }
        ("ui", "rail") => {
            let on = value.as_bool().ok_or("expected bool")?;
            guard.ui.rail = on;
            services.rail_visible.store(on, std::sync::atomic::Ordering::SeqCst);
        }
        ("window", "move_step") => {
            guard.window.move_step = value.as_u64().ok_or("expected number")? as u32;
        }
        ("window", "resize_step") => {
            guard.window.resize_step = value.as_u64().ok_or("expected number")? as u32;
        }
        _ => return Err(format!("unknown config key: {section}.{key}")),
        }
        guard
            .validate()
            .map_err(|e| format!("{e} (изменение не сохранено)"))?;
        guard.clone()
    };
    out.save("config.toml").map_err(|e| e.to_string())?;
    emit_indicator(&app, &services, &out);
    if section == "prompts" {
        sync_ctx_builder(&app, &services, None).await;
    }
    Ok(())
}

/// Применить контекст к активной встрече (из выпадашки «Контекст» оверлея).
#[tauri::command]
pub async fn context_apply(
    app: AppHandle,
    services: State<'_, Services>,
    state: State<'_, Store>,
    id: String,
) -> Result<(), String> {
    let mid = services.active_meeting.lock().await.clone();
    if let Some(mid) = &mid {
        state
            .lock()
            .await
            .set_meeting_context(mid, &id)
            .map_err(|e| e.to_string())?;
    }
    sync_ctx_builder(&app, &services, mid).await;
    Ok(())
}

/// context_id активной встречи (для отметки в выпадашке).
#[tauri::command]
pub async fn context_current(services: State<'_, Services>) -> Result<String, String> {
    let mid = services.active_meeting.lock().await.clone();
    let Some(mid) = mid else { return Ok(String::new()) };
    let store = services.store.lock().await;
    let ctx_id = store
        .list_meetings()
        .ok()
        .and_then(|ms| ms.into_iter().find(|m| m.id == mid))
        .and_then(|m| m.context_id)
        .unwrap_or_default();
    Ok(ctx_id)
}

#[tauri::command]
pub async fn set_hotkey(
    cfg: State<'_, ConfigState>,
    action: String,
    accel: String,
) -> Result<(), String> {
    const ACTIONS: &[&str] = &[
        "manual",
        "hide",
        "click_through",
        "mute",
        "record",
        "screenshot_full",
        "screenshot_region",
        "tts",
    ];
    if !ACTIONS.contains(&action.as_str()) {
        return Err(format!("unknown action: {action}"));
    }
    if !accel.trim().is_empty() {
        accel
            .parse::<tauri_plugin_global_shortcut::Shortcut>()
            .map_err(|e| e.to_string())?;
    }
    let mut guard = cfg.write().map_err(|e| e.to_string())?;
    match action.as_str() {
        "manual" => guard.hotkeys.manual = accel,
        "hide" => guard.hotkeys.hide = accel,
        "click_through" => guard.hotkeys.click_through = accel,
        "mute" => guard.hotkeys.mute = accel,
        "record" => guard.hotkeys.record = accel,
        "screenshot_full" => guard.hotkeys.screenshot_full = accel,
        "screenshot_region" => guard.hotkeys.screenshot_region = accel,
        "tts" => guard.hotkeys.tts = accel,
        _ => unreachable!(),
    }
    let out = guard.clone();
    drop(guard);
    out.save("config.toml").map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_audio_settings(
    cfg: State<'_, ConfigState>,
    source: String,
    mode: String,
    mic_device: Option<String>,
) -> Result<(), String> {
    if !matches!(source.as_str(), "system+mic" | "system" | "mic") {
        return Err(format!("invalid source: {source}"));
    }
    if !matches!(mode.as_str(), "vad" | "manual") {
        return Err(format!("invalid mode: {mode}"));
    }
    let mut guard = cfg.write().map_err(|e| e.to_string())?;
    guard.audio.source = source;
    guard.audio.mode = mode;
    guard.audio.mic_device = mic_device;
    let out = guard.clone();
    drop(guard);
    out.save("config.toml").map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<String>, String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let mut names = Vec::new();
    for dev in host.input_devices().map_err(|e| e.to_string())? {
        if let Ok(name) = dev.name() {
            names.push(name);
        }
    }
    Ok(names)
}

#[tauri::command]
pub async fn tts_play_last(
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
) -> Result<(), String> {
    let cfg = cfg.read().map_err(|e| e.to_string())?.clone();
    crate::tts::play_last(&services, &cfg).await
}

#[tauri::command]
pub async fn tts_speak(
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
    text: String,
) -> Result<(), String> {
    let cfg = cfg.read().map_err(|e| e.to_string())?.clone();
    crate::tts::speak(&services, &cfg, &text).await
}

#[tauri::command]
pub async fn tts_auto_get(cfg: State<'_, ConfigState>) -> Result<bool, String> {
    let cfg = cfg.read().map_err(|e| e.to_string())?;
    Ok(cfg.tts.mode == "auto")
}

#[tauri::command]
pub async fn ui_set(
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
    key: String,
    value: bool,
) -> Result<(), String> {
    if key == "rail" {
        services.rail_visible.store(value, Ordering::SeqCst);
        let mut guard = cfg.write().map_err(|e| e.to_string())?;
        guard.ui.rail = value;
        let out = guard.clone();
        drop(guard);
        out.save("config.toml").map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tts_set_mode(
    app: AppHandle,
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
    mode: String,
) -> Result<(), String> {
    if !matches!(mode.as_str(), "off" | "auto" | "hotkey") {
        return Err(format!("invalid tts mode: {mode}"));
    }
    let mut guard = cfg.write().map_err(|e| e.to_string())?;
    if mode != "off" && guard.tts.api_key.is_empty() {
        return Err("tts api_key is empty: set [tts] api_key in config.toml first".into());
    }
    guard.tts.mode = mode;
    let out = guard.clone();
    drop(guard);
    out.save("config.toml").map_err(|e| e.to_string())?;
    emit_indicator(&app, &services, &cfg_of(&app).unwrap_or_default());
    Ok(())
}

#[tauri::command]
pub async fn tts_auto_set(
    app: AppHandle,
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
    on: bool,
) -> Result<(), String> {
    let mode = if on { "auto" } else { "off" };
    tts_set_mode(app, services, cfg, mode.into()).await
}

// --- 022 overlay ---

async fn active_meeting(services: &Arc<AppServices>) -> Result<String, String> {
    services
        .active_meeting
        .lock()
        .await
        .clone()
        .ok_or_else(|| "нет активной встречи: начните запись из главного окна".into())
}

#[tauri::command]
pub async fn chats_list(services: State<'_, Services>) -> Result<Vec<ChatRow>, String> {
    let meeting = active_meeting(&services).await?;
    services
        .store
        .lock()
        .await
        .list_chats(&meeting)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_create(services: State<'_, Services>) -> Result<String, String> {
    let meeting = active_meeting(&services).await?;
    let row = services
        .store
        .lock()
        .await
        .create_chat(&meeting)
        .map_err(|e| e.to_string())?;
    Ok(row.id)
}

#[tauri::command]
pub async fn chat_switch(services: State<'_, Services>, id: String) -> Result<(), String> {
    services.orch.set_active_chat(id);
    Ok(())
}

#[tauri::command]
pub async fn notes_list(state: State<'_, Store>) -> Result<Vec<NoteRow>, String> {
    state.lock().await.notes_list().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn note_get(state: State<'_, Store>, id: String) -> Result<NoteRow, String> {
    state.lock().await.note_get(&id).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct SttView {
    pub provider: String,
    pub model: String,
    pub mode: String,
    pub language: String,
}

#[derive(Serialize)]
pub struct UiView {
    pub ui: UiSection,
    pub chat: ChatSection,
    pub llm_model: String,
    pub stt_mode: String,
    pub stt_model: String,
    pub stt_language: String,
    pub tts_mode: String,
}

#[tauri::command]
pub async fn stt_get(cfg: State<'_, ConfigState>) -> Result<SttView, String> {
    let c = cfg.read().map_err(|e| e.to_string())?;
    Ok(SttView {
        provider: c.stt.provider.clone(),
        model: c.stt.model.clone(),
        mode: c.audio.mode.clone(),
        language: c.stt.language.clone(),
    })
}

#[tauri::command]
pub async fn stt_set(
    app: AppHandle,
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
    provider: String,
    model: String,
    mode: String,
    language: String,
) -> Result<(), String> {
    if !matches!(provider.as_str(), "groq" | "deepgram" | "soniox") {
        return Err(format!("invalid provider: {provider}"));
    }
    if model.trim().is_empty() {
        return Err("model is required".into());
    }
    if !matches!(mode.as_str(), "vad" | "manual") {
        return Err(format!("invalid mode: {mode}"));
    }
    if !matches!(language.as_str(), "auto" | "ru" | "en") {
        return Err(format!("invalid language: {language}"));
    }
    let changed = cfg
        .read()
        .map_err(|e| e.to_string())
        .map(|g| g.stt.provider != provider || g.stt.model != model || g.audio.mode != mode)?;
    cfg.write()
        .map_err(|e| e.to_string())
        .map(|mut guard| {
            guard.stt.provider = provider;
            guard.stt.model = model;
            guard.audio.mode = mode;
            guard.stt.language = language;
            guard.clone()
        })?
        .save("config.toml")
        .map_err(|e| e.to_string())?;

    // live-перезапуск пайплайна, чтобы смена провайдера/модели/режима
    // применилась без обновления встречи
    if changed && services.pipeline.lock().await.is_some() {
        if let Some(meeting_id) = services.active_meeting.lock().await.clone() {
            {
                let mut p = services.pipeline.lock().await;
                if let Some(h) = p.as_mut() {
                    pipeline_stop(h, &services.audio).await;
                }
            }
            // WASAPI loopback не успевает освободить устройство сразу после stop;
            // пауза до перезапуска, иначе захват падает с "device no longer available".
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            let handle = pipeline_start(app.clone(), services.store.clone(), meeting_id.clone())
                .await
                .map_err(|e| e.to_string())?;
            *services.pipeline.lock().await = Some(handle);
            services.recording.store(true, Ordering::SeqCst);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn ui_get(cfg: State<'_, ConfigState>) -> Result<UiView, String> {
    let c = cfg.read().map_err(|e| e.to_string())?;
    Ok(UiView {
        ui: c.ui.clone(),
        chat: c.chat.clone(),
        llm_model: c.llm.model.clone(),
        stt_mode: c.audio.mode.clone(),
        stt_model: c.stt.model.clone(),
        stt_language: c.stt.language.clone(),
        tts_mode: c.tts.mode.clone(),
    })
}

#[tauri::command]
pub async fn go_home(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.hide();
    }
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub async fn auto_answers_set(
    app: AppHandle,
    services: State<'_, Services>,
    on: bool,
) -> Result<(), String> {
    services.auto.store(on, Ordering::SeqCst);
    services.orch.set_auto(on);
    emit_indicator(&app, &services, &cfg_of(&app).unwrap_or_default());
    Ok(())
}

#[tauri::command]
pub async fn auto_answers_get(services: State<'_, Services>) -> Result<bool, String> {
    Ok(services.auto.load(Ordering::SeqCst))
}

#[tauri::command]
pub async fn search_set(
    services: State<'_, Services>,
    cfg: State<'_, ConfigState>,
    on: bool,
) -> Result<(), String> {
    let mut guard = cfg.write().map_err(|e| e.to_string())?;
    guard.llm.search_enabled = on;
    let out = guard.clone();
    drop(guard);
    out.save("config.toml").map_err(|e| e.to_string())?;
    services.orch.set_search(on);
    Ok(())
}

#[tauri::command]
pub async fn notes_rag_set(services: State<'_, Services>, on: bool) -> Result<(), String> {
    services.notes_rag.store(on, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn ctx_reset(services: State<'_, Services>) -> Result<(), String> {
    services.orch.reset_active();
    Ok(())
}
