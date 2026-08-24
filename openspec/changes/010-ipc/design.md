# Design: IPC Wiring

## 1. Обновление workspace Cargo.toml

Добавить в `[workspace.dependencies]`:

```toml
raw-window-handle = "0.6"
tauri-plugin-global-shortcut = "2"
```

## 2. apps/desktop/Cargo.toml

```toml
[dependencies]
tauri.workspace = true
tauri-plugin-global-shortcut.workspace = true
raw-window-handle.workspace = true
tokio.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
chrono.workspace = true
uuid.workspace = true
engine-config = { path = "../../crates/engine-config" }
engine-audio = { path = "../../crates/engine-audio" }
engine-vad = { path = "../../crates/engine-vad" }
engine-stt = { path = "../../crates/engine-stt" }
engine-dialogue = { path = "../../crates/engine-dialogue" }
engine-context = { path = "../../crates/engine-context" }
engine-llm = { path = "../../crates/engine-llm" }
engine-orchestrator = { path = "../../crates/engine-orchestrator" }
```

## 3. src/main.rs (composition root)

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use engine_audio::{AudioEngine, AudioEvent};
use engine_config::Config;
use engine_context::ContextBuilder;
use engine_dialogue::{Assembler, Speaker, Transcript as DTranscript};
use engine_llm::LlmClient;
use engine_orchestrator::{OrchEvent, Orchestrator};
use engine_stt::{AudioSegment, SttProcessor};
use engine_vad::{Segmenter, VadProcessor};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use uuid::Uuid;

#[tauri::command]
async fn manual_trigger(
    state: tauri::State<'_, Arc<Orchestrator>>,
    note: Option<String>,
) {
    state.manual(note);
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let cfg_path = "config.toml";
            let (cfg_lock, _cfg_events) = engine_config::ConfigWatcher::start(cfg_path)?;
            let cfg = cfg_lock.read().unwrap().clone();

            // --- components ---
            let audio = AudioEngine::new();
            let vad_i = VadProcessor::new("silero_vad.onnx")?;
            let vad_c = VadProcessor::new("silero_vad.onnx")?;
            let (mut seg_i, mut seg_i_rx) = Segmenter::new(vad_i, cfg.vad.silence_ms, cfg.vad.max_segment_ms);
            let (mut seg_c, mut seg_c_rx) = Segmenter::new(vad_c, cfg.vad.silence_ms, cfg.vad.max_segment_ms);
            let (stt, mut stt_rx) = SttProcessor::new(cfg.stt.api_key.clone(), 3);

            let ctx = ContextBuilder::new(cfg.prompts.system.clone(), cfg.prompts.persona.clone(), 8000);
            let llm = LlmClient::new(
                cfg.llm.base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".into()),
                cfg.llm.api_key.clone(), cfg.llm.model.clone(), cfg.llm.temperature, cfg.llm.max_tokens,
            );
            let orch = Arc::new(Orchestrator::new(ctx, llm, cfg.orchestrator.debounce_ms, cfg.orchestrator.min_words));

            app.manage(orch.clone());

            // --- audio -> segmenters ---
            let mut audio_rx = audio.subscribe();
            tokio::spawn(async move {
                while let Ok(ev) = audio_rx.recv().await {
                    let res = match ev {
                        AudioEvent::SystemData(d) => seg_i.process_chunk(&d).await,
                        AudioEvent::MicData(d) => seg_c.process_chunk(&d).await,
                    };
                    if let Err(e) = res { tracing::error!("vad: {e}"); }
                }
            });
            audio.start_system_capture()?;
            audio.start_mic_capture()?;

            // --- segmenters -> stt (с тегом lane) ---
            let lanes: Arc<Mutex<HashMap<Uuid, Speaker>>> = Arc::new(Mutex::new(HashMap::new()));
            let lanes_i = lanes.clone();
            let stt_i = stt; // одна очередь на обе lane
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(s) = seg_i_rx.recv() => {
                            let id = Uuid::new_v4();
                            lanes_i.lock().await.insert(id, Speaker::Interviewer);
                            let _ = stt_i.process_segment(AudioSegment { id, audio: s.audio, duration_ms: s.duration_ms }).await;
                        }
                        Some(s) = seg_c_rx.recv() => {
                            let id = Uuid::new_v4();
                            lanes_i.lock().await.insert(id, Speaker::Candidate);
                            let _ = stt_i.process_segment(AudioSegment { id, audio: s.audio, duration_ms: s.duration_ms }).await;
                        }
                        else => break,
                    }
                }
            });

            // --- stt -> assembler -> orchestrator + UI ---
            let assembler = Arc::new(Mutex::new(Assembler::new()));
            let orch2 = orch.clone();
            let handle2 = handle.clone();
            tokio::spawn(async move {
                while let Some((seg, res)) = stt_rx.recv().await {
                    let speaker = lanes.lock().await.remove(&seg.id).unwrap_or(Speaker::Interviewer);
                    let Ok(t) = res else { continue };
                    let tr = DTranscript {
                        speaker, text: t.text, start_time: chrono::Utc::now(), duration_ms: seg.duration_ms,
                    };
                    let turn = assembler.lock().await.process_transcript(tr).await.ok().flatten();
                    if let Some(turn) = turn {
                        let _ = handle2.emit("turn", serde_json::json!({
                            "speaker": format!("{:?}", turn.speaker), "text": turn.text
                        }));
                        orch2.on_turn(turn);
                    }
                }
            });

            // --- orchestrator -> UI ---
            let mut orch_rx = orch.subscribe();
            let handle3 = handle.clone();
            tokio::spawn(async move {
                while let Ok(ev) = orch_rx.recv().await {
                    match ev {
                        OrchEvent::Token(t) => { let _ = handle3.emit("answer_token", t); }
                        OrchEvent::Done => { let _ = handle3.emit("answer_done", ()); }
                        OrchEvent::Skipped => { let _ = handle3.emit("answer_skipped", ()); }
                        OrchEvent::Error(e) => { let _ = handle3.emit("status", format!("error: {e}")); }
                        OrchEvent::Status(s) => { let _ = handle3.emit("status", s); }
                    }
                }
            });

            // --- hotkeys ---
            let orch3 = orch.clone();
            let win = app.get_webview_window("main").unwrap();
            use tauri_plugin_global_shortcut::ShortcutState;
            app.global_shortcut().on_shortcut("Ctrl+Shift+Space", move |_app, _sc, ev| {
                if ev.state == ShortcutState::Pressed { orch3.manual(None); }
            })?;
            let win2 = app.get_webview_window("main").unwrap();
            app.global_shortcut().on_shortcut("Ctrl+Shift+H", move |_app, _sc, ev| {
                if ev.state == ShortcutState::Pressed {
                    if win2.is_visible().unwrap_or(false) { let _ = win2.hide(); } else { let _ = win2.show(); }
                }
            })?;
            let _ = win;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![manual_trigger])
        .run(tauri::generate_context!())?;

    Ok(())
}
```

## 4. capabilities/default.json

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "global-shortcut:default"
  ]
}
```

## Рассмотрено и отклонено
- **Отдельный engine-ipc crate:** отклонено — wiring живёт в main.rs, проще для MVP
- **Tauri state для assembler:** отклонено — assembler в closure-таске
