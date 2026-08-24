pub fn strip_code(md: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in md.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            out.push_str(line);
        }
    }
    out
}

pub fn split_sentences(buf: &str, flush: bool) -> (Vec<String>, String) {
    let mut out = Vec::new();
    let mut start = 0;
    let b = buf.as_bytes();
    for i in 0..b.len() {
        if matches!(b[i], b'.' | b'!' | b'?' | b';' | b'\n') {
            let mut end = i + 1;
            while end < b.len() && (b[end] as char).is_whitespace() {
                end += 1;
            }
            let piece = buf[start..end].trim();
            if piece.chars().count() >= 2 {
                out.push(piece.to_string());
            }
            start = end;
        }
    }
    let rem = buf[start..].to_string();
    if flush {
        let t = rem.trim();
        if !t.is_empty() {
            out.push(t.to_string());
        }
        (out, String::new())
    } else {
        (out, rem)
    }
}

pub fn resample_linear_f32(src: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || src.is_empty() {
        return src.to_vec();
    }
    let ratio = to as f64 / from as f64;
    let n = (src.len() as f64 * ratio) as usize;
    (0..n)
        .map(|i| {
            let pos = i as f64 / ratio;
            let i0 = pos as usize;
            let i1 = (i0 + 1).min(src.len() - 1);
            let t = (pos - i0 as f64) as f32;
            src[i0] * (1.0 - t) + src[i1] * t
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_code_removes_fences() {
        let md = "Идея.\n```python\nprint(1)\n```\nКонец.";
        let out = strip_code(md);
        assert!(!out.contains("print(1)"));
        assert!(out.contains("Идея."));
        assert!(out.contains("Конец."));
    }

    #[test]
    fn split_by_sentence() {
        let (sents, rem) = split_sentences("Привет! Как де", false);
        assert_eq!(sents, vec!["Привет!"]);
        assert_eq!(rem, "Как де");
    }

    #[test]
    fn flush_emits_tail() {
        let (sents, rem) = split_sentences("без точки", true);
        assert_eq!(sents, vec!["без точки"]);
        assert_eq!(rem, "");
    }

    #[test]
    fn resample_length_ratio() {
        let src: Vec<f32> = (0..1000).map(|i| i as f32).collect();
        let out = resample_linear_f32(&src, 22050, 44100);
        let d = out.len() as i64 - 2000;
        assert!(d.abs() <= 2, "len={}", out.len());
    }

    #[test]
    fn resample_same_rate() {
        let src: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let out = resample_linear_f32(&src, 22050, 22050);
        assert_eq!(out, src);
    }
}