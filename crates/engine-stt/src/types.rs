use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct AudioSegment {
    pub id: uuid::Uuid,
    pub audio: Vec<f32>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Transcript {
    pub text: String,
    #[serde(default)]
    pub duration: f32,
    #[serde(default)]
    pub avg_logprob: f32,
    #[serde(default)]
    pub no_speech_prob: f32,
    /// Единая шкала уверенности [0..1], заполняется клиентом
    /// (groq/deepgram маппят свою метрику сами).
    #[serde(default = "def_confidence")]
    pub confidence: f32,
    /// Длительность сегмента в мс (выставляется из VAD-сегмента в пайплайне).
    #[serde(default)]
    pub duration_ms: u64,
}

fn def_confidence() -> f32 {
    0.6
}

impl Transcript {
    /// True when the segment is most likely a hallucination on silence or
    /// background noise, not real speech. Работает в единой шкале confidence [0..1].
    pub fn likely_hallucination(&self) -> bool {
        const HARD_CONF: f32 = 0.35; // ниже — явная неуверенность
        const SHORT_MS: u64 = 800; // короткие реплики дропаем только при средней неуверенности
        const SHORT_CONF: f32 = 0.6;

        let t = self.text.trim();
        if t.is_empty() {
            return true; // шум/тишина
        }
        if Self::blacklisted(t) {
            return true;
        }
        if self.confidence < HARD_CONF {
            return true;
        }
        if self.duration_ms < SHORT_MS && self.confidence < SHORT_CONF {
            return true;
        }
        false
    }

    /// Типовые whisper-галлюцинации/артефакты на тишине.
    fn blacklisted(text: &str) -> bool {
        let norm = text
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();
        let norm = norm.trim();
        matches!(
            norm,
            "" | "you"
                | "thank you"
                | "thank you for watching"
                | "thanks for watching"
                | "please subscribe"
                | "subtitles"
                | "music"
                | "applause"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(text: &str, conf: f32, dur_ms: u64) -> Transcript {
        Transcript {
            text: text.into(),
            duration: dur_ms as f32 / 1000.0,
            avg_logprob: -(1.0 - conf),
            no_speech_prob: 0.0,
            confidence: conf,
            duration_ms: dur_ms,
        }
    }

    #[test]
    fn real_speech_kept() {
        // deepgram: общая уверенность 0.85, реальная фраза из лога
        assert!(!tx("I was in parliament.", 0.85, 1984).likely_hallucination());
        assert!(!tx("I don't know what she's looking to.", 0.82, 2400).likely_hallucination());
    }

    #[test]
    fn short_weak_dropped() {
        assert!(tx("hmm", 0.4, 500).likely_hallucination());
    }

    #[test]
    fn short_confident_kept() {
        assert!(!tx("yes", 0.9, 700).likely_hallucination());
    }

    #[test]
    fn low_conf_dropped() {
        assert!(tx("some words", 0.2, 3000).likely_hallucination());
    }

    #[test]
    fn empty_dropped() {
        assert!(tx("", 0.9, 2000).likely_hallucination());
    }

    #[test]
    fn blacklist_dropped() {
        assert!(tx("Thank you for watching.", 0.95, 4000).likely_hallucination());
        assert!(tx("Please subscribe", 0.9, 3000).likely_hallucination());
    }

    #[test]
    fn groq_mapping() {
        // groq (whisper): avg_logprob -> confidence
        let real = tx("real speech", 0.7, 2500); // -0.3 -> 0.7
        assert!(!real.likely_hallucination());
        let low = tx("noise", 0.1, 2500); // -0.9 -> 0.1
        assert!(low.likely_hallucination());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub type SegmentResult = Result<Transcript, crate::SttError>;
pub type TranscriptStream = tokio::sync::mpsc::Receiver<(AudioSegment, SegmentResult)>;
