use crate::{AudioSegment, CircuitBreaker, SegmentResult, SttClient, SttError, Transcript};
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::time::{sleep, Duration};

pub struct SttQueue {
    sender: mpsc::Sender<AudioSegment>,
}

impl SttQueue {
    pub fn new(
        client: Arc<SttClient>,
        max_concurrency: usize,
        max_queue_size: usize,
    ) -> (Self, mpsc::Receiver<(AudioSegment, SegmentResult)>) {
        let (input_sender, mut input_receiver) = mpsc::channel::<AudioSegment>(max_queue_size);
        let (output_sender, output_receiver) = mpsc::channel(max_queue_size);

        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let circuit_breaker = Arc::new(CircuitBreaker::new(5, 30));

        let circuit_breaker_clone = circuit_breaker.clone();

        tokio::spawn(async move {
            while let Some(segment) = input_receiver.recv().await {
                let client = client.clone();
                let semaphore = semaphore.clone();
                let output_sender = output_sender.clone();
                let circuit_breaker = circuit_breaker_clone.clone();

                tokio::spawn(async move {
                    let permit = semaphore.acquire().await;

                    let result = match permit {
                        Ok(_) => {
                            Self::transcribe_with_retries(&client, &segment, &circuit_breaker).await
                        }
                        Err(_) => Err(SttError::QueueFull),
                    };

                    let _ = output_sender.send((segment, result)).await;
                });
            }
        });

        (
            Self {
                sender: input_sender,
            },
            output_receiver,
        )
    }

    pub async fn submit(&self, segment: AudioSegment) -> Result<(), SttError> {
        self.sender
            .try_send(segment)
            .map_err(|_| SttError::QueueFull)
    }

    async fn transcribe_with_retries(
        client: &SttClient,
        segment: &AudioSegment,
        circuit_breaker: &CircuitBreaker,
    ) -> Result<Transcript, SttError> {
        if !circuit_breaker.allow_request().await {
            return Err(SttError::CircuitOpen);
        }

        let mut delay_ms = 100;
        let mut last_error: Option<SttError> = None;

        for attempt in 0..3 {
            match client.transcribe(&segment.audio).await {
                Ok(transcript) => {
                    circuit_breaker.record_success().await;
                    return Ok(transcript);
                }
                Err(e) => {
                    circuit_breaker.record_failure().await;

                    if matches!(e, SttError::Authentication) {
                        return Err(e);
                    }

                    last_error = Some(e);

                    if attempt < 2 {
                        sleep(Duration::from_millis(delay_ms)).await;
                        delay_ms *= 2;
                    }
                }
            }
        }

        Err(SttError::MaxRetriesExceeded {
            last_error: last_error.map(|e| e.to_string()).unwrap_or_default(),
        })
    }
}
