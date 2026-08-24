/// Грубая оценка токенов без внешних библиотек: ~4 символа на токен.
pub fn estimate_tokens(s: &str) -> usize {
    s.chars().count() / 4 + 1
}
