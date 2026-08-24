use crate::stealth::apply_affinity;
use engine_audio::AudioEngine;
use engine_config::Config;
use engine_dialogue::{Assembler, Speaker, Transcript as DTranscript};
use engine_orchestrator::{OrchEvent, Orchestrator};
use engine_rtstt::{RtEvent, RtSession, SonioxConfig, CHUNK};
use engine_store::{ReplayEvent, ReplayLogger, SessionStore};
use engine_stt::{AudioSegment, SttProcessor};
use engine_vad::{Segmenter, VadProcessor};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

pub struct AppServices {
    pub store: Arc<Mutex<SessionStore>>,
    pub audio: Arc<Mutex<AudioEngine>>,
    pub orch: Arc<Orchestrator>,
    pub pipeline: Mutex<Option<PipelineHandle>>,
    pub recording: Arc<AtomicBool>,
    pub tts: Mutex<crate::tts::TtsState>,
    pub active_meeting: Arc<Mutex<Option<String>>>,
    pub auto: Arc<AtomicBool>,
    pub notes_rag: Arc<AtomicBool>,
    pub rail_visible: Arc<AtomicBool>,
}

pub struct PipelineHandle {
    stop_tx: Option<broadcast::Sender<()>>,
}

fn emit_to_overlay(app: &AppHandle, event: &str, payload: impl serde::Serialize + Clone) {
    let res = match app.get_webview_window("overlay") {
        Some(w) => w.emit(event, payload),
        None => app.emit(event, payload),
    };
    match res {
        Ok(_) => tracing::debug!("emit {event} ok"),
        Err(e) => tracing::error!("emit {event} failed: {e}"),
    }
}

fn s16le_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
        .collect()
}

async fn persist_turn(
    store: &Arc<Mutex<SessionStore>>,
    session_id: &str,
    meeting_id: &str,
    turn: &engine_dialogue::Turn,
) {
    let spk = format!("{:?}", turn.speaker);
    let _ = store.lock().await.insert_turn(
        session_id,
        &spk,
        &turn.text,
        &turn.start_time.to_rfc3339(),
        &turn.end_time.to_rfc3339(),
    );
    let _ = store.lock().await.bump_messages(meeting_id, 1);
}

/// Команды фоновой записи. Forwarder'ы шлют их в unbounded-канал и не ждут
/// ответа, поэтому блокирующий I/O (rusqlite, файл) не держит их на критическом пути.
enum WriteCmd {
    Transcript { lane: String, text: String },
    Turn {
        speaker: String,
        text: String,
        start: String,
        end: String,
        session: String,
        meeting: String,
    },
}

/// Фоновая запись: одна за раз, без contention между лейнами.
async fn write_task(
    mut rx: tokio::sync::mpsc::UnboundedReceiver<WriteCmd>,
    store: Arc<Mutex<SessionStore>>,
    logger: Arc<Mutex<ReplayLogger>>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            WriteCmd::Transcript { lane, text } => {
                let _ = logger.lock().await.log(&ReplayEvent::Transcript {
                    id: Uuid::new_v4().to_string(),
                    lane,
                    text,
                });
            }
            WriteCmd::Turn {
                speaker,
                text,
                start,
                end,
                session,
                meeting,
            } => {
                let _ = logger.lock().await.log(&ReplayEvent::Turn {
                    speaker: speaker.clone(),
                    text: text.clone(),
                });
                let st = store.lock().await;
                let _ = st.insert_turn(&session, &speaker, &text, &start, &end);
                let _ = st.bump_messages(&meeting, 1);
            }
        }
    }
}

pub async fn start(
    app: AppHandle,
    store: Arc<Mutex<SessionStore>>,
    meeting_id: String,
) -> anyhow::Result<PipelineHandle> {
    let cfg = Config::load("config.toml")?;
    let services = app.state::<Arc<AppServices>>();
    let audio = services.audio.clone();
    let orch = services.orch.clone();

    let session_id = Uuid::new_v4().to_string();
    store
        .lock()
        .await
        .start_session(&session_id, &serde_json::to_string(&cfg)?)?;
    let logger = Arc::new(Mutex::new(ReplayLogger::open(PathBuf::from(format!(
        "sessions/{session_id}"
    )))?));

    // фоновая запись (replay + store) вне критического пути forwarder'а
    let (write_tx, write_rx) = tokio::sync::mpsc::unbounded_channel::<WriteCmd>();
    tokio::spawn(write_task(write_rx, store.clone(), logger.clone()));

    let (stop_tx, _) = broadcast::channel::<()>(4);

    // --- capture: останавливаем старые потоки, затем запускаем заново ---
    {
        let mut audio_guard = audio.lock().await;
        audio_guard.stop();
        let sys_rx = audio_guard.start_system_capture()?;
        let mic_rx = audio_guard.start_mic_capture(cfg.audio.mic_device.as_deref())?;

        if cfg.stt.provider == "soniox" {
            start_soniox(
                app.clone(),
                &cfg,
                orch.clone(),
                session_id.clone(),
                meeting_id.clone(),
                &stop_tx,
                write_tx.clone(),
                sys_rx,
                mic_rx,
            )
            .await?;
        } else {
            start_batch(
                app.clone(),
                &cfg,
                store.clone(),
                orch.clone(),
                logger.clone(),
                session_id.clone(),
                meeting_id.clone(),
                &stop_tx,
                sys_rx,
                mic_rx,
            )
            .await?;
        }
    }

    // --- orchestrator -> UI ---
    let mut orch_rx = orch.subscribe();
    let handle3 = app.clone();
    let services3 = handle3.state::<Arc<AppServices>>().inner().clone();
    let logger_orch = logger.clone();
    let store_orch = store.clone();
    let session_orch = session_id.clone();
    let meeting_orch = meeting_id.clone();
    let mut stop_orch = stop_tx.subscribe();
    tokio::spawn(async move {
        let mut current_gen: u64 = 0;
        let mut gen_start = std::time::Instant::now();
        let mut ttft_ms: u64 = 0;
        loop {
            tokio::select! {
                _ = stop_orch.recv() => break,
                ev = orch_rx.recv() => {
                    let Ok(ev) = ev else { break };
                    match ev {
                        OrchEvent::Token { gen, text } => {
                            if gen != current_gen {
                                continue;
                            }
                            tracing::debug!("orch token");
                            if ttft_ms == 0 {
                                ttft_ms = gen_start.elapsed().as_millis() as u64;
                            }
                            emit_to_overlay(&handle3, "answer_token", text.clone());
                            crate::tts::feed_token(&services3, &text).await;
                        }
                        OrchEvent::Done { gen, text } => {
                            if gen != current_gen {
                                continue;
                            }
                            tracing::info!("orch done");
                            let _ = store_orch.lock().await.insert_answer(
                                &session_orch,
                                "manual",
                                "answered",
                                &text,
                                0,
                                ttft_ms,
                            );
                            let _ = logger_orch.lock().await.log(&ReplayEvent::Answer {
                                outcome: "answered".into(),
                                text: text.clone(),
                                ttft_ms,
                            });
                            let _ = store_orch.lock().await.bump_messages(&meeting_orch, 1);
                            emit_to_overlay(&handle3, "answer_done", ());
                            crate::tts::finish(&services3, &text).await;
                        }
                        OrchEvent::Error { gen, message } => {
                            if gen != current_gen {
                                continue;
                            }
                            tracing::error!("orch error: {message}");
                            let _ = store_orch.lock().await.insert_answer(
                                &session_orch,
                                "manual",
                                "error",
                                &message,
                                0,
                                0,
                            );
                            let _ = logger_orch.lock().await.log(&ReplayEvent::Answer {
                                outcome: "error".into(),
                                text: message.clone(),
                                ttft_ms: 0,
                            });
                            emit_to_overlay(&handle3, "status", format!("error: {message}"));
                            crate::tts::reset(&services3).await;
                        }
                        OrchEvent::Status { gen, state } => {
                            tracing::info!("orch status: {state}");
                            if state == "generating" {
                                current_gen = gen;
                                gen_start = std::time::Instant::now();
                                ttft_ms = 0;
                                crate::tts::reset(&services3).await;
                                let cfg = handle3
                                    .state::<Arc<RwLock<Config>>>()
                                    .read()
                                    .map(|g| g.clone())
                                    .unwrap_or_default();
                                crate::tts::start_auto(&services3, &cfg).await;
                            }
                            emit_to_overlay(&handle3, "status", state);
                        }
                    }
                }
            }
        }
    });

    if cfg.ui.protection {
        let _ = apply_affinity(&app.get_webview_window("overlay").ok_or_else(|| {
            anyhow::anyhow!("overlay window not found")
        })?);
    }

    Ok(PipelineHandle {
        stop_tx: Some(stop_tx),
    })
}

#[allow(clippy::too_many_arguments)]
async fn start_soniox(
    app: AppHandle,
    cfg: &Config,
    orch: Arc<Orchestrator>,
    session_id: String,
    meeting_id: String,
    stop_tx: &broadcast::Sender<()>,
    write_tx: tokio::sync::mpsc::UnboundedSender<WriteCmd>,
    sys_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
    mic_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
) -> anyhow::Result<()> {
    let hints: Vec<String> = cfg
        .stt
        .language_hints
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let soniox_cfg = SonioxConfig {
        api_key: cfg.stt.soniox_api_key.clone(),
        model: cfg.stt.model.clone(),
        language_hints: hints,
        utterance_idle_ms: cfg.stt.utterance_idle_ms,
        max_utterance_chars: cfg.stt.max_utterance_chars,
        ws_url: "wss://stt-rt.soniox.com/transcribe-websocket".into(),
    };

    let session_i = engine_rtstt::start(soniox_cfg.clone())?;
    let session_c = engine_rtstt::start(soniox_cfg)?;

    spawn_lane(
        app.clone(),
        cfg,
        orch.clone(),
        session_id.clone(),
        meeting_id.clone(),
        stop_tx,
        write_tx.clone(),
        "I",
        Speaker::Interviewer,
        sys_rx,
        session_i,
    )?;
    spawn_lane(
        app.clone(),
        cfg,
        orch,
        session_id,
        meeting_id,
        stop_tx,
        write_tx,
        "C",
        Speaker::Candidate,
        mic_rx,
        session_c,
    )?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn spawn_lane(
    app: AppHandle,
    cfg: &Config,
    orch: Arc<Orchestrator>,
    session_id: String,
    meeting_id: String,
    stop_tx: &broadcast::Sender<()>,
    write_tx: tokio::sync::mpsc::UnboundedSender<WriteCmd>,
    lane: &'static str,
    speaker: Speaker,
    bytes_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
    session: RtSession,
) -> anyhow::Result<()> {
    let audio_mode = cfg.audio.mode.clone();
    let audio_source = cfg.audio.source.clone();
    let recording = app.state::<Arc<AppServices>>().recording.clone();

    // chunker: gate -> режем по CHUNK -> шлём в сессию; пишем дамп lane PCM
    let dump_path = PathBuf::from(format!("sessions/{session_id}/audio/lane_{lane}.pcm"));
    let mut stop_chunk = stop_tx.subscribe();
    let mut bytes_rx = bytes_rx;
    let audio_tx = session.audio_tx.clone();
    let is_mic = lane == "C";
    let mut dump = File::create(&dump_path)?;
    tokio::spawn(async move {
        let mut buf: Vec<u8> = Vec::with_capacity(CHUNK * 2);
        loop {
            tokio::select! {
                _ = stop_chunk.recv() => break,
                b = bytes_rx.recv() => {
                    let Some(b) = b else { break };
                    if !engine_orchestrator::gate(
                        &audio_mode,
                        recording.load(Ordering::SeqCst),
                        &audio_source,
                        is_mic,
                    ) {
                        continue;
                    }
                    if dump.write_all(&b).is_err() {
                        break;
                    }
                    buf.extend_from_slice(&b);
                    while buf.len() >= CHUNK {
                        let piece: Vec<u8> = buf.drain(..CHUNK).collect();
                        if audio_tx.send(piece).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // forwarder: события сессии -> UI / assembler -> turn -> store + orch
    let assembler = Arc::new(Mutex::new(Assembler::new()));
    let mut stop_ev = stop_tx.subscribe();
    let mut events = session.events;
    let handle = session.handle;
    tokio::spawn(async move {
        let _keep_alive = handle;
        loop {
            tokio::select! {
                _ = stop_ev.recv() => break,
                ev = events.recv() => {
                    let Some(ev) = ev else { break };
                    match ev {
                        RtEvent::Partial { text, utt_id } => {
                            tracing::debug!("stt_partial emit {lane}: {text}");
                            emit_to_overlay(
                                &app,
                                "stt_partial",
                                serde_json::json!({ "lane": lane, "text": text, "utt_id": utt_id }),
                            );
                            if lane == "I" {
                                orch.on_partial(text.clone());
                            }
                        }
                        RtEvent::Utterance { text, utt_id } => {
                            orch.on_partial(String::new());
                            tracing::info!("STT {lane}: {text:?}");
                            let t0 = std::time::Instant::now();
                            let _ = write_tx.send(WriteCmd::Transcript {
                                lane: lane.into(),
                                text: text.clone(),
                            });
                            let d1 = t0.elapsed();
                            let tr = DTranscript {
                                speaker,
                                text,
                                start_time: chrono::Utc::now(),
                                duration_ms: 0,
                            };
                            let turn = assembler
                                .lock()
                                .await
                                .process_transcript(tr)
                                .await
                                .ok()
                                .flatten();
                            let d2 = t0.elapsed();
                            if let Some(turn) = turn {
                                let _ = write_tx.send(WriteCmd::Turn {
                                    speaker: format!("{:?}", turn.speaker),
                                    text: turn.text.clone(),
                                    start: turn.start_time.to_rfc3339(),
                                    end: turn.end_time.to_rfc3339(),
                                    session: session_id.clone(),
                                    meeting: meeting_id.clone(),
                                });
                                let d3 = t0.elapsed();
                                emit_to_overlay(
                                    &app,
                                    "turn",
                                    serde_json::json!({
                                        "speaker": format!("{:?}", turn.speaker),
                                        "text": turn.text,
                                        "id": utt_id,
                                    }),
                                );
                                let d4 = t0.elapsed();
                                orch.on_turn(turn);
                                tracing::debug!(
                                    lane = %lane,
                                    log_us = d1.as_micros(),
                                    asm_us = d2.as_micros().saturating_sub(d1.as_micros()),
                                    store_us = d3.as_micros().saturating_sub(d2.as_micros()),
                                    emit_us = d4.as_micros().saturating_sub(d3.as_micros()),
                                    total_us = d4.as_micros(),
                                    "forwarder timing"
                                );
                            }
                        }
                        RtEvent::Error(msg) => {
                            tracing::error!("soniox {lane} error: {msg}");
                            emit_to_overlay(&app, "status", format!("soniox {lane}: {msg}"));
                        }
                        RtEvent::Closed => break,
                    }
                }
            }
        }
    });

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn start_batch(
    app: AppHandle,
    cfg: &Config,
    store: Arc<Mutex<SessionStore>>,
    orch: Arc<Orchestrator>,
    logger: Arc<Mutex<ReplayLogger>>,
    session_id: String,
    meeting_id: String,
    stop_tx: &broadcast::Sender<()>,
    sys_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
    mic_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
) -> anyhow::Result<()> {
    let vad_i = VadProcessor::new("silero_vad.onnx")?;
    let vad_c = VadProcessor::new("silero_vad.onnx")?;
    let (mut seg_i, mut seg_i_rx) =
        Segmenter::new(vad_i, cfg.vad.silence_ms, cfg.vad.max_segment_ms);
    let (mut seg_c, mut seg_c_rx) =
        Segmenter::new(vad_c, cfg.vad.silence_ms, cfg.vad.max_segment_ms);
    let (stt, mut stt_rx) = SttProcessor::with_provider(
        &cfg.stt.provider,
        cfg.stt.api_key.clone(),
        cfg.stt.model.clone(),
        cfg.stt.language.clone(),
        3,
    )?;

    // --- vad states -> UI ---
    let mut seg_i_states = seg_i.subscribe_states();
    let mut seg_c_states = seg_c.subscribe_states();
    let emit_handle = app.clone();
    let mut stop_vad = stop_tx.subscribe();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = stop_vad.recv() => break,
                Ok(s) = seg_i_states.recv() => {
                    emit_to_overlay(&emit_handle, "vad", serde_json::json!({"lane": "I", "state": format!("{s:?}")}));
                }
                Ok(s) = seg_c_states.recv() => {
                    emit_to_overlay(&emit_handle, "vad", serde_json::json!({"lane": "C", "state": format!("{s:?}")}));
                }
                else => break,
            }
        }
    });

    // --- audio bytes -> f32 -> segmenters ---
    let mut sys_rx = sys_rx;
    let mut mic_rx = mic_rx;
    let mut stop_audio = stop_tx.subscribe();
    let recording = app.state::<Arc<AppServices>>().recording.clone();
    let audio_source = cfg.audio.source.clone();
    let audio_mode = cfg.audio.mode.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = stop_audio.recv() => break,
                b = sys_rx.recv() => {
                    let Some(b) = b else { break };
                    if engine_orchestrator::gate(&audio_mode, recording.load(Ordering::SeqCst), &audio_source, false) {
                        let data = s16le_to_f32(&b);
                        if let Err(e) = seg_i.process_chunk(&data).await {
                            tracing::error!("vad I: {e}");
                        }
                    }
                }
                b = mic_rx.recv() => {
                    let Some(b) = b else { break };
                    if engine_orchestrator::gate(&audio_mode, recording.load(Ordering::SeqCst), &audio_source, true) {
                        let data = s16le_to_f32(&b);
                        if let Err(e) = seg_c.process_chunk(&data).await {
                            tracing::error!("vad C: {e}");
                        }
                    }
                }
            }
        }
    });

    // --- segmenters -> stt (с тегом lane) ---
    let lanes: Arc<Mutex<HashMap<Uuid, Speaker>>> = Arc::new(Mutex::new(HashMap::new()));
    let lanes_i = lanes.clone();
    let stt_i = stt;
    let logger_seg = logger.clone();
    let mut stop_seg = stop_tx.subscribe();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = stop_seg.recv() => break,
                Some(s) = seg_i_rx.recv() => {
                    tracing::info!("VAD segment interviewer: {}ms", s.duration_ms);
                    let id = Uuid::new_v4();
                    lanes_i.lock().await.insert(id, Speaker::Interviewer);
                    let _ = logger_seg.lock().await.save_segment_wav(&id.to_string(), &s.audio);
                    let _ = logger_seg.lock().await.log(&ReplayEvent::Segment {
                        id: id.to_string(),
                        lane: "I".into(),
                        duration_ms: s.duration_ms,
                    });
                    let _ = stt_i.process_segment(AudioSegment { id, audio: s.audio, duration_ms: s.duration_ms }).await;
                }
                Some(s) = seg_c_rx.recv() => {
                    tracing::info!("VAD segment candidate: {}ms", s.duration_ms);
                    let id = Uuid::new_v4();
                    lanes_i.lock().await.insert(id, Speaker::Candidate);
                    let _ = logger_seg.lock().await.save_segment_wav(&id.to_string(), &s.audio);
                    let _ = logger_seg.lock().await.log(&ReplayEvent::Segment {
                        id: id.to_string(),
                        lane: "C".into(),
                        duration_ms: s.duration_ms,
                    });
                    let _ = stt_i.process_segment(AudioSegment { id, audio: s.audio, duration_ms: s.duration_ms }).await;
                }
                else => break,
            }
        }
    });

    // --- stt -> assembler -> orchestrator + UI ---
    let assembler = Arc::new(Mutex::new(Assembler::new()));
    let orch2 = orch.clone();
    let handle2 = app.clone();
    let logger_stt = logger.clone();
    let store_stt = store.clone();
    let session_stt = session_id;
    let meeting_stt = meeting_id;
    let mut stop_stt = stop_tx.subscribe();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = stop_stt.recv() => break,
                rec = stt_rx.recv() => {
                    let Some((seg, res)) = rec else { break };
                    let speaker = lanes.lock().await.remove(&seg.id).unwrap_or(Speaker::Interviewer);
                    let Ok(mut t) = res else {
                        tracing::error!("stt failed for segment {}: {:?}", seg.id, res.err());
                        continue;
                    };
                    t.duration_ms = seg.duration_ms;
                    if t.likely_hallucination() {
                        tracing::info!(
                            "dropping likely STT hallucination {:?} ({}ms, conf {:.2})",
                            t.text,
                            seg.duration_ms,
                            t.confidence
                        );
                        continue;
                    }
                    tracing::info!("STT {}: {:?}", format!("{:?}", speaker), t.text);
                    let lane = match speaker {
                        Speaker::Interviewer => "I",
                        Speaker::Candidate => "C",
                    };
                    let _ = logger_stt.lock().await.log(&ReplayEvent::Transcript {
                        id: seg.id.to_string(),
                        lane: lane.into(),
                        text: t.text.clone(),
                    });
                    let tr = DTranscript {
                        speaker,
                        text: t.text,
                        start_time: chrono::Utc::now(),
                        duration_ms: seg.duration_ms,
                    };
                    let turn = assembler.lock().await.process_transcript(tr).await.ok().flatten();
                    if let Some(turn) = turn {
                        emit_to_overlay(
                            &handle2,
                            "turn",
                            serde_json::json!({
                                "speaker": format!("{:?}", turn.speaker),
                                "text": turn.text
                            }),
                        );
                        let _ = logger_stt.lock().await.log(&ReplayEvent::Turn {
                            speaker: format!("{:?}", turn.speaker),
                            text: turn.text.clone(),
                        });
                        persist_turn(&store_stt, &session_stt, &meeting_stt, &turn).await;
                        orch2.on_turn(turn);
                    }
                }
            }
        }
    });

    Ok(())
}

pub async fn stop(h: &mut PipelineHandle, audio: &Arc<Mutex<AudioEngine>>) {
    if let Some(tx) = h.stop_tx.take() {
        let _ = tx.send(());
        tracing::info!("pipeline stopped");
    }
    audio.lock().await.stop();
}