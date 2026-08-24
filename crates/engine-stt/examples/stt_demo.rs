use engine_stt::{AudioSegment, SttProcessor};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let api_key = std::env::var("GROQ_API_KEY")
        .map_err(|_| anyhow::anyhow!("GROQ_API_KEY environment variable is required"))?;

    let (processor, mut receiver) = SttProcessor::new(api_key, 3)?;

    let handler = tokio::spawn(async move {
        while let Some((segment, result)) = receiver.recv().await {
            match result {
                Ok(transcript) => {
                    tracing::info!("Transcript for {}: {}", segment.id, transcript.text);
                }
                Err(e) => {
                    tracing::error!("Failed to transcribe {}: {}", segment.id, e);
                }
            }
        }
    });

    for i in 0..10 {
        let segment = AudioSegment {
            id: uuid::Uuid::new_v4(),
            audio: vec![0.0f32; 16000],
            duration_ms: 1000,
        };
        tracing::info!("Submitting segment {i}");
        processor.process_segment(segment).await?;
    }

    drop(processor);
    handler.await?;

    Ok(())
}
