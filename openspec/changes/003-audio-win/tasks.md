# Tasks: Windows Audio Capture

## Phase 1: Dependencies

- [x] 1.1 Обновить `crates/engine-audio/Cargo.toml` добавить зависимости из design.md §1
  verify: `cargo build -p engine-audio` проходит

## Phase 2: Error Types

- [x] 2.1 Создать `crates/engine-audio/src/error.rs` из design.md §2
  verify: `cargo build -p engine-audio` проходит

## Phase 3: Stream Types

- [x] 3.1 Создать `crates/engine-audio/src/stream.rs` с AudioEvent enum из design.md §5
  verify: `cargo build -p engine-audio` проходит

## Phase 4: Resampler

- [x] 4.1 Создать `crates/engine-audio/src/resampler.rs` из design.md §3
  verify: `cargo build -p engine-audio` проходит

## Phase 5: Audio Engine

- [x] 5.1 Создать `crates/engine-audio/src/capture.rs` с AudioEngine struct из design.md §4
  verify: `cargo build -p engine-audio` проходит

- [x] 5.2 Реализовать `new()` и `subscribe()` методы из design.md §4
  verify: `cargo build -p engine-audio` проходит

- [x] 5.3 Реализовать `start_system_capture()` метод из design.md §4
  verify: `cargo build -p engine-audio` проходит

- [x] 5.4 Реализовать `start_mic_capture()` метод из design.md §4
  verify: `cargo build -p engine-audio` проходит

- [x] 5.5 Реализовать `to_mono()` helper из design.md §4
  verify: `cargo build -p engine-audio` проходит

## Phase 6: Public API

- [x] 6.1 Обновить `crates/engine-audio/src/lib.rs` с pub use из design.md §1
  verify: `cargo build -p engine-audio` проходит

## Phase 7: Tests

- [x] 7.1 Создать `crates/engine-audio/tests/audio_tests.rs`
  verify: файл создан

- [x] 7.2 Тест `captures_system_audio` (scenario из specs) - интеграционный тест
  verify: `cargo test -p engine-audio captures_system_audio` проходит (manual run с audio device)

- [x] 7.3 Тест `captures_microphone` (scenario из specs) - интеграционный тест
  verify: `cargo test -p engine-audio captures_microphone` проходит (manual run с mic)

- [x] 7.4 Тест `dual_lane_capture` (scenario из specs) - интеграционный тест
  verify: `cargo test -p engine-audio dual_lane_capture` проходит

- [x] 7.5 Тест `resamples_to_16khz` (scenario из specs) - unit тест для resampler
  verify: `cargo test -p engine-audio resamples_to_16khz` проходит

- [x] 7.6 Тест `async_stream` (scenario из specs) - async тест
  verify: `cargo test -p engine-audio async_stream` проходит

## Phase 8: Integration Test

- [x] 8.1 Создать `examples/audio_capture.rs` из design.md §6
  verify: `cargo run -p engine-audio --example audio_capture` запускается

- [x] 8.2 Запустить example и проверить что аудио захватывается (manual)
  verify: логи показывают "System: N samples" и "Mic: N samples"

## Phase 9: Validation

- [x] 9.1 Запустить `cargo clippy -p engine-audio --all-targets -- -D warnings`
  verify: выход 0

- [x] 9.2 Запустить `cargo test -p engine-audio`
  verify: все тесты проходят

- [x] 9.3 Запустить `cargo build -p engine-audio --release`
  verify: выход 0

## STOP Protocol

Если:
- `cpal::host_from_id(cpal::HostId::Wasapi)` падает → WASAPI не поддерживается, проверить Windows version
- `build_input_stream` падает с StreamConfig error → проверить что channels=1 в config
- `rubato` resampler не инициализируется → проверить что input_rate != output_rate

Не пытаться добавить поддержку других audio backends или Linux/macOS. Остановиться и спросить.
