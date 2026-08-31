use crate::{is_echo, Dialogue, DialogueError, Speaker, Transcript, Turn};
use chrono::Duration;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const FILLERS: [&str; 6] = ["ок", "okay", "хорошо", "ага", "угу", "спасибо"];

pub struct Assembler {
    buffer: BinaryHeap<Reverse<Transcript>>,
    turns: Vec<Turn>,
    summary: String,
    merge_threshold_ms: i64,
    dedup_threshold_secs: i64,
    summary_threshold: usize,
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            buffer: BinaryHeap::new(),
            turns: Vec::new(),
            summary: String::new(),
            merge_threshold_ms: 500,
            dedup_threshold_secs: 2,
            summary_threshold: 16,
        }
    }

    pub fn with_params(
        merge_threshold_ms: i64,
        dedup_threshold_secs: i64,
        summary_threshold: usize,
    ) -> Self {
        Self {
            buffer: BinaryHeap::new(),
            turns: Vec::new(),
            summary: String::new(),
            merge_threshold_ms,
            dedup_threshold_secs,
            summary_threshold,
        }
    }

    pub async fn process_transcript(
        &mut self,
        transcript: Transcript,
    ) -> Result<Option<Turn>, DialogueError> {
        self.buffer.push(Reverse(transcript));
        self.process_buffer().await
    }

    async fn process_buffer(&mut self) -> Result<Option<Turn>, DialogueError> {
        let mut first_created = None;

        while let Some(Reverse(transcript)) = self.buffer.pop() {
            if self.is_garbage(&transcript) {
                tracing::debug!(text = %transcript.text, "Filtered garbage");
                continue;
            }

            if self.is_duplicate(&transcript) {
                tracing::debug!(text = %transcript.text, "Filtered duplicate");
                continue;
            }

            if transcript.speaker == Speaker::Candidate {
                let candidate = Turn {
                    speaker: transcript.speaker,
                    text: transcript.text.clone(),
                    start_time: transcript.start_time,
                    end_time: transcript.start_time
                        + Duration::milliseconds(transcript.duration_ms as i64),
                    typed: false,
                };
                if self.is_echo_drop(&candidate) {
                    tracing::debug!(text = %transcript.text, "Filtered cross-lane echo");
                    continue;
                }
            }

            let idx = self.insertion_index(&transcript);

            if idx > 0 {
                let can_merge = {
                    let prev = &self.turns[idx - 1];
                    self.can_merge(prev, &transcript)
                };

                if can_merge {
                    let prev = &mut self.turns[idx - 1];
                    prev.text.push(' ');
                    prev.text.push_str(&transcript.text);
                    prev.end_time = transcript.start_time
                        + Duration::milliseconds(transcript.duration_ms as i64);
                    tracing::debug!(text = %transcript.text, "Merged with previous turn");
                    continue;
                }
            }

            let turn = Turn {
                speaker: transcript.speaker,
                text: transcript.text,
                start_time: transcript.start_time,
                end_time: transcript.start_time
                    + Duration::milliseconds(transcript.duration_ms as i64),
                typed: false,
            };

            self.turns.insert(idx, turn.clone());
            tracing::info!(speaker = ?turn.speaker, text = %turn.text, "New turn");

            if first_created.is_none() {
                first_created = Some(turn);
            }

            if self.turns.len() >= self.summary_threshold {
                self.generate_summary().await?;
            }
        }

        Ok(first_created)
    }

    fn insertion_index(&self, transcript: &Transcript) -> usize {
        self.turns.partition_point(|t| {
            (t.start_time, t.speaker.lane()) < (transcript.start_time, transcript.speaker.lane())
        })
    }

    fn is_garbage(&self, transcript: &Transcript) -> bool {
        let text = transcript.text.trim().to_lowercase();
        FILLERS.contains(&text.as_str())
    }

    /// C-реплика отбрасывается, если она — эхо последних реплик интервьюера.
    fn is_echo_drop(&self, candidate: &Turn) -> bool {
        let mut recent_i = self
            .turns
            .iter()
            .rev()
            .filter(|t| t.speaker == Speaker::Interviewer)
            .take(2);
        recent_i.any(|prev| is_echo(prev, candidate))
    }

    fn is_duplicate(&self, transcript: &Transcript) -> bool {
        let idx = self.insertion_index(transcript);
        if idx == 0 {
            return false;
        }

        let prev = &self.turns[idx - 1];

        if prev.speaker != transcript.speaker {
            return false;
        }

        if prev.text.trim().to_lowercase() != transcript.text.trim().to_lowercase() {
            return false;
        }

        let time_diff = transcript.start_time.signed_duration_since(prev.end_time);
        time_diff.num_seconds().abs() <= self.dedup_threshold_secs
    }

    fn can_merge(&self, prev: &Turn, transcript: &Transcript) -> bool {
        if prev.speaker != transcript.speaker {
            return false;
        }

        let pause_ms = transcript
            .start_time
            .signed_duration_since(prev.end_time)
            .num_milliseconds();

        (0..self.merge_threshold_ms).contains(&pause_ms)
    }

    async fn generate_summary(&mut self) -> Result<(), DialogueError> {
        if self.turns.len() < 4 {
            return Ok(());
        }

        let to_summarize: Vec<_> = self.turns.drain(..4).collect();

        let summary_text = to_summarize
            .iter()
            .map(|t| format!("{:?}: {}", t.speaker, t.text))
            .collect::<Vec<_>>()
            .join(" ");

        if self.summary.is_empty() {
            self.summary = summary_text;
        } else {
            self.summary.push(' ');
            self.summary.push_str(&summary_text);
        }

        tracing::info!(summary = %self.summary, "Generated summary");

        Ok(())
    }

    pub fn get_dialogue(&self) -> Dialogue {
        Dialogue {
            turns: self.turns.clone(),
            summary: self.summary.clone(),
            total_turns: self.turns.len(),
        }
    }

    pub fn get_recent_turns(&self, count: usize) -> Vec<Turn> {
        let start = self.turns.len().saturating_sub(count);
        self.turns[start..].to_vec()
    }
}
