use crate::pipeline::AppServices;
use engine_config::Config;
use engine_tts::cartesia::{start_session, CartesiaConfig, TtsCmd, TtsOut, TtsSession};
use engine_tts::feeder::SentenceFeeder;
use engine_tts::player::Player;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::AbortHandle;

pub struct TtsSessionHandle {
    pub cmd: mpsc::Sender<TtsCmd>,
    pub handle: AbortHandle,
}

pub struct TtsState {
    pub player: Player,
    pub session: Option<TtsSessionHandle>,
    pub feeder: SentenceFeeder,
    pub last_answer: String,
    pub playing: Arc<AtomicBool>,
}

impl TtsState {
    pub fn new() -> Self {
        Self {
            player: Player::new(),
            session: None,
            feeder: SentenceFeeder::new(),
            last_answer: String::new(),
            playing: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for TtsState {
    fn default() -> Self {
        Self::new()
    }
}

fn cartesia_cfg(cfg: &Config) -> Option<CartesiaConfig> {
    if cfg.tts.api_key.is_empty() {
        return None;
    }
    Some(CartesiaConfig {
        api_key: cfg.tts.api_key.clone(),
        model_id: cfg.tts.model_id.clone(),
        voice_id: cfg.tts.voice_id.clone(),
        sample_rate: cfg.tts.sample_rate,
    })
}

fn spawn_reader(player: Player, mut out: mpsc::Receiver<TtsOut>, rate: u32, playing: Arc<AtomicBool>) {
    tauri::async_runtime::spawn(async move {
        while let Some(o) = out.recv().await {
            match o {
                TtsOut::Pcm(f) => {
                    if let Err(e) = player.push(f, rate) {
                        tracing::error!("tts player: {e}");
                        break;
                    }
                }
                TtsOut::Done => break,
                TtsOut::Error(e) => {
                    tracing::error!("tts session: {e}");
                    break;
                }
            }
        }
        playing.store(false, Ordering::SeqCst);
    });
}

fn begin_session(g: &mut TtsState, session: TtsSession, rate: u32) {
    let TtsSession { cmd, out, handle } = session;
    g.playing.store(true, Ordering::SeqCst);
    spawn_reader(g.player.clone(), out, rate, g.playing.clone());
    g.session = Some(TtsSessionHandle { cmd, handle });
}

/// Aborts current session, clears queue, resets state (new answer / stop).
pub async fn reset(services: &Arc<AppServices>) {
    let mut g = services.tts.lock().await;
    if let Some(s) = g.session.take() {
        s.handle.abort();
    }
    g.playing.store(false, Ordering::SeqCst);
    g.player.clear();
    g.feeder = SentenceFeeder::new();
    g.last_answer.clear();
}

/// Starts a streaming session when the next generation begins (mode=auto).
pub async fn start_auto(services: &Arc<AppServices>, cfg: &Config) {
    if cfg.tts.mode != "auto" {
        return;
    }
    let mut g = services.tts.lock().await;
    let Some(c) = cartesia_cfg(cfg) else {
        tracing::warn!("tts mode=auto but api_key is empty");
        return;
    };
    match start_session(c) {
        Ok(session) => {
            let rate = cfg.tts.sample_rate;
            begin_session(&mut g, session, rate);
        }
        Err(e) => tracing::error!("tts start: {e}"),
    }
}

/// Feed a token into the streaming session (mode=auto).
pub async fn feed_token(services: &Arc<AppServices>, text: &str) {
    let mut g = services.tts.lock().await;
    g.last_answer.push_str(text);
    let sents = g.feeder.push_token(text);
    if let Some(sess) = g.session.as_ref() {
        for s in sents {
            let _ = sess.cmd.send(TtsCmd::Text(s)).await;
        }
    }
}

/// Finish the streaming session (mode=auto): trailing sentence + flush.
pub async fn finish(services: &Arc<AppServices>, text: &str) {
    let mut g = services.tts.lock().await;
    g.last_answer = text.to_string();
    let sents = g.feeder.finish();
    if let Some(sess) = g.session.as_ref() {
        for s in sents {
            let _ = sess.cmd.send(TtsCmd::Text(s)).await;
        }
        let _ = sess.cmd.send(TtsCmd::Flush).await;
    }
}

/// Ctrl+T / speaker button: toggle playing the last answer.
pub async fn play_last(services: &Arc<AppServices>, cfg: &Config) -> Result<(), String> {
    if cfg.tts.mode == "off" {
        return Ok(());
    }
    let mut g = services.tts.lock().await;
    if g.playing.load(Ordering::SeqCst) {
        if let Some(s) = g.session.take() {
            s.handle.abort();
        }
        g.playing.store(false, Ordering::SeqCst);
        g.player.clear();
        return Ok(());
    }
    let Some(c) = cartesia_cfg(cfg) else {
        return Err("tts api_key is not set in config.toml".into());
    };
    let session = start_session(c).map_err(|e| e.to_string())?;
    let rate = cfg.tts.sample_rate;
    begin_session(&mut g, session, rate);
    g.feeder = SentenceFeeder::new();
    let last = g.last_answer.clone();
    let sents = g.feeder.push_token(&last);
    let tail = g.feeder.finish();
    let cmd = g.session.as_ref().unwrap().cmd.clone();
    drop(g);
    for s in sents.into_iter().chain(tail) {
        let _ = cmd.send(TtsCmd::Text(s)).await;
    }
    let _ = cmd.send(TtsCmd::Flush).await;
    Ok(())
}

/// Speaker button on an AI bubble: speak arbitrary text.
pub async fn speak(services: &Arc<AppServices>, cfg: &Config, text: &str) -> Result<(), String> {
    if cfg.tts.mode == "off" {
        return Ok(());
    }
    let mut g = services.tts.lock().await;
    if g.playing.load(Ordering::SeqCst) {
        if let Some(s) = g.session.take() {
            s.handle.abort();
        }
        g.playing.store(false, Ordering::SeqCst);
        g.player.clear();
    }
    let Some(c) = cartesia_cfg(cfg) else {
        return Err("tts api_key is not set in config.toml".into());
    };
    let session = start_session(c).map_err(|e| e.to_string())?;
    let rate = cfg.tts.sample_rate;
    begin_session(&mut g, session, rate);
    g.feeder = SentenceFeeder::new();
    let sents = g.feeder.push_token(text);
    let tail = g.feeder.finish();
    let cmd = g.session.as_ref().unwrap().cmd.clone();
    drop(g);
    for s in sents.into_iter().chain(tail) {
        let _ = cmd.send(TtsCmd::Text(s)).await;
    }
    let _ = cmd.send(TtsCmd::Flush).await;
    Ok(())
}