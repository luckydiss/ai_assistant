use engine_config::Config;
use engine_tts::cartesia::{start_session, CartesiaConfig, TtsCmd, TtsOut};
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("tts_probe=info,engine_tts=debug")
        .init();
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let cfg = Config::load("config.toml")?;
        if cfg.tts.mode == "off" || cfg.tts.api_key.is_empty() {
            anyhow::bail!("set [tts] mode and api_key in config.toml first");
        }
        let c = CartesiaConfig {
            api_key: cfg.tts.api_key.clone(),
            model_id: cfg.tts.model_id.clone(),
            voice_id: cfg.tts.voice_id.clone(),
            sample_rate: cfg.tts.sample_rate,
        };
        let mut s = start_session(c)?;
        let t0 = Instant::now();
        let mut first_pcm: Option<Instant> = None;
        let mut total = 0usize;
        let mut file = std::fs::File::create("probe.pcm")?;
        use std::io::Write;

        s.cmd.send(TtsCmd::Text("Привет! Как дела?".into())).await?;
        s.cmd.send(TtsCmd::Flush).await?;

        loop {
            let out = match tokio::time::timeout(std::time::Duration::from_secs(15), s.out.recv()).await {
                Ok(Some(o)) => o,
                Ok(None) => {
                    tracing::info!("channel closed");
                    break;
                }
                Err(_) => {
                    tracing::info!("timeout waiting for tts output");
                    break;
                }
            };
            match out {
                TtsOut::Pcm(f) => {
                    if first_pcm.is_none() {
                        first_pcm = Some(Instant::now());
                    }
                    total += f.len();
                    tracing::info!("pcm chunk {} samples (first={first_pcm:?})", f.len());
                    let bytes: Vec<u8> = f.iter().flat_map(|x| x.to_le_bytes()).collect();
                    file.write_all(&bytes)?;
                }
                TtsOut::Done => {
                    tracing::info!("got done");
                    break;
                }
                TtsOut::Error(e) => {
                    tracing::error!("tts error: {e}");
                    anyhow::bail!("tts error: {e}");
                }
            }
        }
        let t_first = first_pcm.map(|t| t.duration_since(t0)).unwrap_or_default();
        tracing::info!("first pcm after {t_first:?}, total {} samples, session {}", total, s.handle.is_finished());
        Ok(())
    })
}