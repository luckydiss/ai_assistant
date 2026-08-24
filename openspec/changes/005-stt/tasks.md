# Tasks: Speech-to-Text

## Phase 1: Dependencies

- [x] 1.1 Обновить `crates/engine-stt/Cargo.toml` добавить зависимости из design.md §1
  verify: `cargo build -p engine-stt` проходит

## Phase 2: Error Types

- [x] 2.1 Создать `crates/engine-stt/src/error.rs` из design.md §2
  verify: `cargo build -p engine-stt` проходит

## Phase 3: Types

- [x] 3.1 Создать `crates/engine-stt/src/types.rs` с AudioSegment, Transcript, CircuitState из design.md §3
  verify: `cargo build -p engine-stt` проходит

## Phase 4: Groq Client

- [x] 4.1 Создать `crates/engine-stt/src/client.rs` с GroqClient struct из design.md §4
  verify: `cargo build -p engine-stt` проходит

- [x] 4.2 Реализовать `new()` метод из design.md §4
  verify: `cargo build -p engine-stt` проходит

- [x] 4.3 Реализовать `transcribe()` метод из design.md §4
  verify: `cargo build -p engine-stt` проходит

- [x] 4.4 Реализовать `encode_wav()` helper из design.md §4
  verify: `cargo build -p engine-stt` проходит

## Phase 5: Circuit Breaker

- [x] 5.1 Создать `crates/engine-stt/src/circuit.rs` с CircuitBreaker struct из design.md §5
  verify: `cargo build -p engine-stt` проходит

- [x] 5.2 Реализовать `new()` метод из design.md §5
  verify: `cargo build -p engine-stt` проходит

- [x] 5.3 Реализовать `allow_request()` метод из design.md §5
  verify: `cargo build -p engine-stt` проходит

- [x] 5.4 Реализовать `record_success()` метод из design.md §5
  verify: `cargo build -p engine-stt` проходит

- [x] 5.5 Реализовать `record_failure()` метод из design.md §5
  verify: `cargo build -p engine-stt` проходит

## Phase 6: Queue

- [x] 6.1 Создать `crates/engine-stt/src/queue.rs` с SttQueue struct из design.md §6
  verify: `cargo build -p engine-stt` проходит

- [x] 6.2 Реализовать `new()` метод из design.md §6
  verify: `cargo build -p engine-stt` проходит

- [x] 6.3 Реализовать `submit()` метод из design.md §6
  verify: `cargo build -p engine-stt` проходит

- [x] 6.4 Реализовать `transcribe_with_retries()` helper из design.md §6
  verify: `cargo build -p engine-stt` проходит

## Phase 7: Processor

- [x] 7.1 Создать `crates/engine-stt/src/processor.rs` с SttProcessor struct из design.md §7
  verify: `cargo build -p engine-stt` проходит

- [x] 7.2 Реализовать `new()` метод из design.md §7
  verify: `cargo build -p engine-stt` проходит

- [x] 7.3 Реализовать `process_segment()` метод из design.md §7
  verify: `cargo build -p engine-stt` проходит

## Phase 8: Public API

- [x] 8.1 Обновить `crates/engine-stt/src/lib.rs` с pub use из design.md §1
  verify: `cargo build -p engine-stt` проходит

## Phase 9: Tests

- [x] 9.1 Создать `crates/engine-stt/tests/stt_tests.rs`
  verify: файл создан

- [x] 9.2 Тест `transcribes_successfully` (scenario из specs) - mock HTTP server
  verify: `cargo test -p engine-stt transcribes_successfully` проходит

- [x] 9.3 Тест `errors_on_invalid_key` (scenario из specs) - mock HTTP 401
  verify: `cargo test -p engine-stt errors_on_invalid_key` проходит

- [x] 9.4 Тест `respects_concurrency_limit` (scenario из specs) - с semaphore
  verify: `cargo test -p engine-stt respects_concurrency_limit` проходит

- [x] 9.5 Тест `rejects_on_overflow` (scenario из specs) - с переполненной очередью
  verify: `cargo test -p engine-stt rejects_on_overflow` проходит

- [x] 9.6 Тест `retries_on_transient_error` (scenario из specs) - mock timeout
  verify: `cargo test -p engine-stt retries_on_transient_error` проходит

- [x] 9.7 Тест `fails_after_max_retries` (scenario из specs) - mock 500 error
  verify: `cargo test -p engine-stt fails_after_max_retries` проходит

- [x] 9.8 Тест `opens_circuit_on_failures` (scenario из specs) - 5 failures
  verify: `cargo test -p engine-stt opens_circuit_on_failures` проходит

- [x] 9.9 Тест `closes_circuit_on_success` (scenario из specs) - с timeout
  verify: `cargo test -p engine-stt closes_circuit_on_success` проходит

- [x] 9.10 Тест `streams_transcripts` (scenario из specs) - async stream
  verify: `cargo test -p engine-stt streams_transcripts` проходит

## Phase 10: Integration Test

- [x] 10.1 Создать `examples/stt_demo.rs` из design.md §8
  verify: `cargo run -p engine-stt --example stt_demo` запускается

- [x] 10.2 Запустить example с реальным Groq API key (manual)
  verify: транскрипты получены корректно

## Phase 11: Validation

- [x] 11.1 Запустить `cargo clippy -p engine-stt --all-targets -- -D warnings`
  verify: выход 0

- [x] 11.2 Запустить `cargo test -p engine-stt`
  verify: все тесты проходят

- [x] 11.3 Запустить `cargo build -p engine-stt --release`
  verify: выход 0

## STOP Protocol

Если:
- `reqwest` multipart form не работает → проверить что hound корректно кодирует WAV
- Circuit breaker не переходит в Open state → проверить логику record_failure с tracing
- Queue переполняется слишком быстро → увеличить max_queue_size или уменьшить producer rate

Не пытаться добавить поддержку других STT провайдеров или local inference. Остановиться и спросить.