use crate::{Speaker, Turn};
use std::collections::HashSet;

/// Сходство по множеству слов (нижний регистр, без пунктуации).
pub fn word_jaccard(a: &str, b: &str) -> f64 {
    let sa = words(a);
    let sb = words(b);
    if sa.is_empty() || sb.is_empty() {
        return 0.0;
    }
    let union_len = sa.union(&sb).count();
    if union_len == 0 {
        return 0.0;
    }
    sa.intersection(&sb).count() as f64 / union_len as f64
}

/// Реплика кандидата C является эхом колонок (системного аудио), если почти
/// совпадает с репликой интервьюера I в окне ±1.5 c.
pub fn is_echo(prev_i: &Turn, cand_c: &Turn) -> bool {
    if prev_i.speaker != Speaker::Interviewer || cand_c.speaker != Speaker::Candidate {
        return false;
    }
    let dt = cand_c
        .start_time
        .signed_duration_since(prev_i.start_time)
        .num_milliseconds()
        .abs();
    dt <= 1500 && word_jaccard(&prev_i.text, &cand_c.text) >= 0.7
}

fn words(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect()
}