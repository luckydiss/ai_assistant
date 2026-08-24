use engine_vad::{Segmenter, VadProcessor};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read_wav(path: &Path) -> Vec<f32> {
    let data = std::fs::read(path).unwrap();
    let data_start = data.windows(4).position(|w| w == b"data").unwrap() + 8;
    data[data_start..]
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
        .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let root = workspace_root();
    std::env::set_var("ORT_DYLIB_PATH", root.join("onnxruntime.dll"));

    let vad = VadProcessor::new(root.join("silero_vad.onnx"))?;
    let (mut segmenter, mut receiver) = Segmenter::new(vad, 600, 7000);

    let segment_handler = tokio::spawn(async move {
        while let Some(segment) = receiver.recv().await {
            tracing::info!(
                "Segment: start={}ms, duration={}ms, samples={}",
                segment.start_time_ms,
                segment.duration_ms,
                segment.audio.len()
            );
        }
    });

    let samples = read_wav(&root.join("aepyx.wav"));
    for chunk in samples.chunks(512) {
        if chunk.len() < 512 {
            break;
        }
        segmenter.process_chunk(chunk).await?;
    }

    drop(segmenter);
    segment_handler.await?;

    Ok(())
}
