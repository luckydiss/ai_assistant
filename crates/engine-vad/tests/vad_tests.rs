use engine_vad::{Segmenter, SpeechSegment, VadProcessor, VadState};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn model_path() -> PathBuf {
    workspace_root().join("silero_vad.onnx")
}

fn wav_path() -> PathBuf {
    workspace_root().join("aepyx.wav")
}

fn init_ort() {
    if std::env::var("ORT_DYLIB_PATH").is_err() {
        std::env::set_var("ORT_DYLIB_PATH", workspace_root().join("onnxruntime.dll"));
    }
}

fn read_wav(path: &Path) -> Vec<f32> {
    let data = std::fs::read(path).unwrap();
    let data_start = data.windows(4).position(|w| w == b"data").unwrap() + 8;
    data[data_start..]
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0)
        .collect()
}

fn vad() -> VadProcessor {
    init_ort();
    VadProcessor::new(model_path()).unwrap()
}

/// Real speech chunks (32ms @16k) extracted from the reference wav.
fn speech_chunks() -> Vec<Vec<f32>> {
    let mut vad = vad();
    let samples = read_wav(&wav_path());
    let mut chunks = Vec::new();
    for frame in samples.chunks(512) {
        if frame.len() < 512 {
            break;
        }
        let r = vad.process_chunk(frame).unwrap();
        if r.speech {
            chunks.push(frame.to_vec());
        }
    }
    assert!(
        !chunks.is_empty(),
        "expected speech chunks in reference wav"
    );
    chunks
}
async fn collect_segments(
    chunks: Vec<Vec<f32>>,
    trailing_silence_chunks: usize,
    silence_ms: u64,
    max_segment_ms: u64,
) -> Vec<SpeechSegment> {
    let (mut segmenter, mut receiver) = Segmenter::new(vad(), silence_ms, max_segment_ms);
    let collector = tokio::spawn(async move {
        let mut segments = Vec::new();
        while let Some(segment) = receiver.recv().await {
            segments.push(segment);
        }
        segments
    });

    for chunk in &chunks {
        segmenter.process_chunk(chunk).await.unwrap();
    }
    for _ in 0..trailing_silence_chunks {
        segmenter.process_chunk(&[0.0; 512]).await.unwrap();
    }

    drop(segmenter);
    collector.await.unwrap()
}

#[tokio::test]
async fn vad_state_sequence() {
    let chunks = speech_chunks();
    let (mut segmenter, mut receiver) = Segmenter::new(vad(), 600, 7000);
    let mut states = segmenter.subscribe_states();
    let collector = tokio::spawn(async move {
        while receiver.recv().await.is_some() {}
    });

    let mut seen = std::collections::HashSet::new();
    for chunk in &chunks {
        segmenter.process_chunk(chunk).await.unwrap();
        while let Ok(s) = states.try_recv() {
            seen.insert(s);
        }
    }
    for _ in 0..25 {
        segmenter.process_chunk(&[0.0; 512]).await.unwrap();
        while let Ok(s) = states.try_recv() {
            seen.insert(s);
        }
    }

    assert!(seen.contains(&VadState::Recording), "expected Recording");
    assert!(seen.contains(&VadState::Waiting), "expected Waiting");
    assert!(seen.contains(&VadState::Paused), "expected Paused after silence");

    drop(segmenter);
    collector.await.unwrap();
}

#[test]
fn loads_model_successfully() {
    let mut vad = vad();
    vad.reset();
}

#[test]
fn errors_on_missing_model() {
    init_ort();
    let result = VadProcessor::new("non_existent_model.onnx");
    assert!(result.is_err());
}

#[test]
fn detects_speech() {
    let mut vad = vad();
    let samples = read_wav(&wav_path());

    let mut detected = false;
    for frame in samples.chunks(512) {
        if frame.len() < 512 {
            break;
        }
        let r = vad.process_chunk(frame).unwrap();
        if r.speech {
            detected = true;
            assert!(r.probability > 0.5);
            break;
        }
    }
    assert!(detected, "expected speech to be detected");
}

#[test]
fn detects_silence() {
    let mut vad = vad();
    let result = vad.process_chunk(&[0.0; 512]).unwrap();
    assert!(!result.speech);
    assert!(result.probability < 0.5);
}

#[tokio::test]
async fn closes_on_silence_600ms() {
    let chunks = speech_chunks();
    let segments = collect_segments(chunks, 25, 600, 7000).await;

    assert!(!segments.is_empty(), "expected at least one segment");
    for s in &segments {
        assert!(!s.audio.is_empty());
        assert!(
            s.duration_ms <= 7000,
            "segment too long: {}ms",
            s.duration_ms
        );
    }
}

#[tokio::test]
async fn splits_long_utterance() {
    let chunks = speech_chunks();
    let segments = collect_segments(chunks, 0, 600, 2000).await;

    assert!(
        segments.len() >= 2,
        "expected >= 2 segments, got {}",
        segments.len()
    );
    for s in &segments {
        assert!(
            s.duration_ms <= 2000,
            "segment too long: {}ms",
            s.duration_ms
        );
    }
}

#[tokio::test]
async fn streams_audio() {
    let chunks = speech_chunks();
    let segments = collect_segments(chunks, 25, 600, 7000).await;

    assert!(!segments.is_empty(), "expected at least one segment");
    assert!(!segments[0].audio.is_empty());
}

#[tokio::test]
async fn preserves_context() {
    let chunks = speech_chunks();
    let take = 50;
    assert!(
        take * 2 * 32 < 7000,
        "test setup: combined audio under 7s limit"
    );

    let mut part1 = chunks.iter().take(take).cloned().collect::<Vec<_>>();
    part1.extend(vec![vec![0.0; 512]; 300 / 32]);
    part1.extend(chunks.iter().take(take).cloned());
    part1.extend(vec![vec![0.0; 512]; 25]);

    let segments = collect_segments(part1, 0, 600, 7000).await;

    assert!(!segments.is_empty(), "expected at least one segment");
    assert!(
        segments[0].duration_ms > take as u64 * 32,
        "expected merged segment, got {}ms (take={})",
        segments[0].duration_ms,
        take
    );
}
