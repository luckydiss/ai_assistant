# Tasks: Voice Activity Detection

## Phase 1: Dependencies

- [x] 1.1 Обновить `crates/engine-vad/Cargo.toml` добавить зависимости из design.md §1
  verify: `cargo build -p engine-vad` проходит

## Phase 2: Error Types

- [x] 2.1 Создать `crates/engine-vad/src/error.rs` из design.md §2
  verify: `cargo build -p engine-vad` проходит

## Phase 3: Types

- [x] 3.1 Создать `crates/engine-vad/src/types.rs` с VadResult и SpeechSegment из design.md §3
  verify: `cargo build -p engine-vad` проходит

## Phase 4: VAD Processor

- [x] 4.1 Создать `crates/engine-vad/src/processor.rs` с VadProcessor struct из design.md §4
  verify: `cargo build -p engine-vad` проходит

- [x] 4.2 Реализовать `new()` метод из design.md §4
  verify: `cargo build -p engine-vad` проходит

- [x] 4.3 Реализовать `process_chunk()` метод из design.md §4
  verify: `cargo build -p engine-vad` проходит

- [x] 4.4 Реализовать `reset()` метод из design.md §4
  verify: `cargo build -p engine-vad` проходит

## Phase 5: Segmenter

- [x] 5.1 Создать `crates/engine-vad/src/segmenter.rs` с Segmenter struct из design.md §5
  verify: `cargo build -p engine-vad` проходит

- [x] 5.2 Реализовать `new()` метод из design.md §5
  verify: `cargo build -p engine-vad` проходит

- [x] 5.3 Реализовать `process_chunk()` метод из design.md §5
  verify: `cargo build -p engine-vad` проходит

- [x] 5.4 Реализовать `should_emit_segment()` helper из design.md §5
  verify: `cargo build -p engine-vad` проходит

- [x] 5.5 Реализовать `emit_segment()` helper из design.md §5
  verify: `cargo build -p engine-vad` проходит

## Phase 6: Public API

- [x] 6.1 Обновить `crates/engine-vad/src/lib.rs` с pub use из design.md §1
  verify: `cargo build -p engine-vad` проходит

## Phase 7: Model Download

- [x] 7.1 Скачать silero_vad.onnx из design.md §7
  verify: файл существует в корне проекта

## Phase 8: Tests

- [x] 8.1 Создать `crates/engine-vad/tests/vad_tests.rs`
  verify: файл создан

- [x] 8.2 Тест `loads_model_successfully` (scenario из specs)
  verify: `cargo test -p engine-vad loads_model_successfully` проходит

- [x] 8.3 Тест `errors_on_missing_model` (scenario из specs)
  verify: `cargo test -p engine-vad errors_on_missing_model` проходит

- [x] 8.4 Тест `detects_speech` (scenario из specs) - с реальным аудио (aepyx.wav)
  verify: `cargo test -p engine-vad detects_speech` проходит

- [x] 8.5 Тест `detects_silence` (scenario из specs) - с тишиной
  verify: `cargo test -p engine-vad detects_silence` проходит

- [x] 8.6 Тест `closes_on_silence_600ms` (scenario из specs) - интеграционный
  verify: `cargo test -p engine-vad closes_on_silence_600ms` проходит

- [x] 8.7 Тест `splits_long_utterance` (scenario из specs) - интеграционный
  verify: `cargo test -p engine-vad splits_long_utterance` проходит

- [x] 8.8 Тест `streams_audio` (scenario из specs) - memory usage check
  verify: `cargo test -p engine-vad streams_audio` проходит

- [x] 8.9 Тест `preserves_context` (scenario из specs) - с короткой паузой
  verify: `cargo test -p engine-vad preserves_context` проходит

## Phase 9: Integration Test

- [x] 9.1 Создать `examples/vad_demo.rs` из design.md §6
  verify: `cargo run -p engine-vad --example vad_demo` запускается

- [x] 9.2 Запустить example с реальным аудио (manual)
  verify: логи показывают корректные сегменты

## Phase 10: Validation

- [x] 10.1 Запустить `cargo clippy -p engine-vad --all-targets -- -D warnings`
  verify: выход 0

- [x] 10.2 Запустить `cargo test -p engine-vad`
  verify: все тесты проходят

- [x] 10.3 Запустить `cargo build -p engine-vad --release`
  verify: выход 0

## STOP Protocol

Если:
- `ort::Session` не загружает модель → проверить что файл silero_vad.onnx существует и валиден
- `process_chunk` падает с tensor shape error → проверить что input всегда 512 samples
- Segmenter не emits сегменты → проверить логику should_emit_segment с tracing

Не пытаться добавить поддержку других VAD моделей или audio форматов. Остановиться и спросить.
