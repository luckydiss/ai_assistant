/// Возвращает payload SSE-строки ("data: X" -> "X"), иначе None.
pub fn parse_sse_line(line: &str) -> Option<&str> {
    line.trim().strip_prefix("data:").map(|p| p.trim())
}

/// Извлекает choices[0].delta.content из JSON-дельты.
pub fn extract_delta(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v["choices"][0]["delta"]["content"]
        .as_str()
        .map(|s| s.to_string())
}
