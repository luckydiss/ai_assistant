use crate::{SpeechSegment, VadProcessor, VadState};
use tokio::sync::{broadcast, mpsc};

const CHUNK_MS: u64 = 32;

pub struct Segmenter {
    vad: VadProcessor,
    silence_ms: u64,
    max_segment_ms: u64,
    buffer: Vec<f32>,
    silence_duration_ms: u64,
    segment_start_ms: u64,
    current_time_ms: u64,
    sender: mpsc::Sender<SpeechSegment>,
    state_tx: broadcast::Sender<VadState>,
    state: VadState,
}

impl Segmenter {
    pub fn new(
        vad: VadProcessor,
        silence_ms: u64,
        max_segment_ms: u64,
    ) -> (Self, mpsc::Receiver<SpeechSegment>) {
        let (sender, receiver) = mpsc::channel(32);
        let (state_tx, _) = broadcast::channel(8);

        let segmenter = Self {
            vad,
            silence_ms,
            max_segment_ms,
            buffer: Vec::with_capacity(112000),
            silence_duration_ms: 0,
            segment_start_ms: 0,
            current_time_ms: 0,
            sender,
            state_tx,
            state: VadState::Waiting,
        };

        (segmenter, receiver)
    }

    pub fn subscribe_states(&self) -> broadcast::Receiver<VadState> {
        self.state_tx.subscribe()
    }

    fn set_state(&mut self, s: VadState) {
        if self.state != s {
            self.state = s;
            let _ = self.state_tx.send(s);
        }
    }

    pub async fn process_chunk(&mut self, audio: &[f32]) -> Result<(), crate::VadError> {
        for chunk in audio.chunks(512) {
            let vad_result = self.vad.process_chunk(chunk)?;

            if !self.buffer.is_empty()
                && self.segment_duration_ms() + CHUNK_MS > self.max_segment_ms
            {
                self.set_state(VadState::Sending);
                self.emit_segment().await?;
                self.set_state(VadState::Waiting);
            }

            if vad_result.speech {
                self.silence_duration_ms = 0;
                if self.buffer.is_empty() {
                    self.set_state(VadState::Recording);
                }
                self.buffer.extend_from_slice(chunk);
            } else {
                self.silence_duration_ms += CHUNK_MS;

                if !self.buffer.is_empty() {
                    self.buffer.extend_from_slice(chunk);
                }
            }

            self.current_time_ms += CHUNK_MS;

            let should_emit = self.should_emit_segment(vad_result.speech);

            if should_emit && !self.buffer.is_empty() {
                self.set_state(VadState::Sending);
                self.emit_segment().await?;
                self.set_state(VadState::Waiting);
            } else if !self.buffer.is_empty() && self.silence_duration_ms > 0 {
                self.set_state(VadState::Paused);
            }
        }

        Ok(())
    }

    fn segment_duration_ms(&self) -> u64 {
        (self.buffer.len() as u64 * 1000) / 16000
    }

    fn should_emit_segment(&self, is_speech: bool) -> bool {
        if self.buffer.is_empty() {
            return false;
        }

        let segment_duration_ms = self.segment_duration_ms();

        if !is_speech && self.silence_duration_ms >= self.silence_ms {
            return true;
        }

        if segment_duration_ms >= self.max_segment_ms {
            return true;
        }

        false
    }

    async fn emit_segment(&mut self) -> Result<(), crate::VadError> {
        let duration_ms = (self.buffer.len() as u64 * 1000) / 16000;

        let segment = SpeechSegment {
            audio: std::mem::take(&mut self.buffer),
            start_time_ms: self.segment_start_ms,
            duration_ms,
        };

        self.sender
            .send(segment)
            .await
            .map_err(|_| crate::VadError::Inference("Channel closed".to_string()))?;

        self.segment_start_ms = self.current_time_ms;
        self.buffer.clear();
        self.silence_duration_ms = 0;

        Ok(())
    }
}
