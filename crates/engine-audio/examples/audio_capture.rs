use engine_audio::AudioEngine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut engine = AudioEngine::new();

    let mut sys_rx = engine.start_system_capture()?;
    let mut mic_rx = engine.start_mic_capture(None)?;

    loop {
        tokio::select! {
            Some(data) = sys_rx.recv() => {
                tracing::debug!("System: {} bytes", data.len());
            }
            Some(data) = mic_rx.recv() => {
                tracing::debug!("Mic: {} bytes", data.len());
            }
        }
    }
}