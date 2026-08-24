use crate::{AudioSegment, SttClient, SttError, SttQueue, TranscriptStream};

pub struct SttProcessor {
    queue: SttQueue,
}

impl SttProcessor {
    pub fn new(
        api_key: String,
        max_concurrency: usize,
    ) -> Result<(Self, TranscriptStream), SttError> {
        Self::with_language(api_key, "auto".to_string(), max_concurrency)
    }

    pub fn with_language(
        api_key: String,
        language: String,
        max_concurrency: usize,
    ) -> Result<(Self, TranscriptStream), SttError> {
        Self::with_provider(
            "groq",
            api_key,
            "whisper-large-v3-turbo".to_string(),
            language,
            max_concurrency,
        )
    }

    pub fn with_provider(
        provider: &str,
        api_key: String,
        model: String,
        language: String,
        max_concurrency: usize,
    ) -> Result<(Self, TranscriptStream), SttError> {
        let client = SttClient::from_config(provider, api_key, model, language)?;
        let (queue, receiver) = SttQueue::new(client, max_concurrency, 100);

        Ok((Self { queue }, receiver))
    }

    pub async fn process_segment(&self, segment: AudioSegment) -> Result<(), SttError> {
        self.queue.submit(segment).await
    }
}
