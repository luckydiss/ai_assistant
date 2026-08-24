# Report: Speech-to-Text (005)

## Summary

Реализован `crates/engine-stt`: Groq API клиент (whisper-large-v3-turbo), асинхронная очередь с ограниченной конкурентностью (Semaphore), retries с exponential backoff (3 попытки), circuit breaker (5 failures → Open, 30s timeout → HalfOpen → Closed), WAV-кодирование через hound. 10/10 тестов проходят на собственноручно написанном tokio mock HTTP-сервере, clippy/fmt/release/workspace OK. Example `stt_demo` запускается и требует `GROQ_API_KEY`.

## Deviations from design.md

1. **`SttProcessor::new` возвращает `Result`:** дизайн фиксирует `(Self, Receiver)` без Result (внутри `.unwrap()` на `Client::builder().build()`). Политика project.md запрещает unwrap в production → `GroqClient::new` возвращает `Result<Self, SttError>`, и `SttProcessor::new` → `Result<(Self, TranscriptStream), SttError>`.

2. **`SttProcessor::new` не позволяет задать base_url (для тестов):** добавлен `GroqClient::with_base_url(api_key, base_url)` — тесты используют mock-сервер вместо реального API. Тесты работают через `SttQueue` + mock-клиент напрямую (не через `SttProcessor`, т.к. тот указывает на реальный Groq).

3. **`submit()` использует `try_send` вместо `send().await`:** дизайн §6 использует `send().await`, который блокируется при полном канале и никогда не вернёт `QueueFull`. Spec требует scenario `rejects_on_overflow` (Err(QueueFull) при переполнении) → `try_send` возвращает `QueueFull` мгновенно.

4. **CircuitBreaker на атомиках вместо Mutex:** дизайн §5 использует вложенные `tokio::sync::Mutex` (state + last_failure) — риск deadlock (вложенные lock) и clippy-замечания. Реализован через `AtomicU8` (state) + `std::sync::Mutex<Option<Instant>>` (last_failure) без вложенных захватов. Добавлен `current_state()` для тестов.

5. **reqwest feature `multipart`:** добавлен в workspace `reqwest = { ..., features = ["multipart"] }` — без него `.multipart(form)` не компилируется.

6. **Поле `circuit_breaker` в SttQueue убрано:** дизайн §6 хранит его в struct, но не использует (breaker создаётся и потребляется внутри `SttQueue::new`). Неиспользуемое поле нарушает `-D warnings`. Circuit breaker остаётся доступен через тесты/внешний код независимо.

7. **Type aliases:** `SegmentResult`, `TranscriptStream` добавлены в types.rs — clippy требует не использовать complex types в сигнатурах.

8. **Mock HTTP server на tokio вместо tiny_http:** для тестов написан минимальный mock-сервер на `tokio::net::TcpListener` (без внешних зависимостей), с отслеживанием активных/максимальных одновременных запросов. tiny_http 0.12 имеет сложный новый config API — отклонён.

9. **`circuit_blocks_when_open` — дополнительный тест:** проверяет, что при открытом breaker запросы отклоняются `Err(CircuitOpen)` БЕЗ HTTP-вызова (кол-во запросов к серверу не растёт). Дизайн/спека требуют это в scenario `opens_circuit_on_failures`, но явный assert на отсутствие HTTP-вызова вынесен отдельно.

10. **Example требует `GROQ_API_KEY`:** `stt_demo` читает ключ из env и возвращает понятную ошибку, если его нет (10.2 manual — не выполнен, нет реального ключа).

11. **`Transcript.duration` — `#[serde(default)]`:** при реальном вызове Groq API ответ не содержит поле `duration` (`{"text":"...","x_groq":{...}}`), из-за чего `response.json()` падал с `missing field "duration"` → `Max retries exceeded`. Поле сделано опциональным (default 0.0). Найдено и исправлено при live-верификации 10.2.

## Verified

- `cargo test -p engine-stt` — 10/10 ok
- `cargo clippy -p engine-stt --all-targets -- -D warnings` — ok
- `cargo build -p engine-stt --release` — ok
- `cargo build --workspace` — ok
- `cargo run -p engine-stt --example stt_demo` — запускается, требует GROQ_API_KEY
- **10.2 (live):** 10 сегментов тишины транскрибированы реальным API (Groq whisper-large-v3-turbo), каждая вернула `Ok(Transcript { text: " Thank you.", duration: 0.0 })`. Валидный API key, WAV-кодирование и multipart работают end-to-end.