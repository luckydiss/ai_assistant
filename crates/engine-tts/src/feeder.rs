use crate::split::{split_sentences, strip_code};

pub struct SentenceFeeder {
    raw: String,
    consumed: usize,
}

impl Default for SentenceFeeder {
    fn default() -> Self {
        Self::new()
    }
}

impl SentenceFeeder {
    pub fn new() -> Self {
        Self {
            raw: String::new(),
            consumed: 0,
        }
    }

    pub fn push_token(&mut self, t: &str) -> Vec<String> {
        self.raw.push_str(t);
        self.drain(false)
    }

    pub fn finish(&mut self) -> Vec<String> {
        let v = self.drain(true);
        self.raw.clear();
        self.consumed = 0;
        v
    }

    fn drain(&mut self, flush: bool) -> Vec<String> {
        let stripped = strip_code(&self.raw);
        if stripped.len() <= self.consumed && !flush {
            return Vec::new();
        }
        let tail = &stripped[self.consumed.min(stripped.len())..];
        let (sents, rem) = split_sentences(tail, flush);
        self.consumed = stripped.len() - rem.len();
        sents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feeder_splits_sentences() {
        let mut f = SentenceFeeder::new();
        let first = f.push_token("Привет! Как де");
        assert_eq!(first, vec!["Привет!"]);
        let second = f.push_token("ла? Код.");
        assert_eq!(second, vec!["Как дела?", "Код."]);
    }

    #[test]
    fn feeder_skips_code() {
        let mut f = SentenceFeeder::new();
        let out = f.push_token("Идея.\n```python\nprint(1)\n```\nКонец.");
        let all = format!("{}{}", out.join(" "), f.finish().join(" "));
        assert!(!all.contains("print(1)"));
        assert!(all.contains("Идея."));
        assert!(all.contains("Конец."));
    }

    #[test]
    fn feeder_flush_tail() {
        let mut f = SentenceFeeder::new();
        assert!(f.push_token("без точки").is_empty());
        assert_eq!(f.finish(), vec!["без точки"]);
        assert!(f.finish().is_empty());
    }

    #[test]
    fn feeder_empty_finish() {
        let mut f = SentenceFeeder::new();
        assert!(f.finish().is_empty());
    }
}