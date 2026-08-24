# Report: Windows Audio Capture (003)

## Summary

Реализован crates/engine-audio: WASAPI loopback захват системного звука и микрофона,
два независимых lane (System/Mic), ресемплинг в 16kHz/mono/f32 через rubato 0.14,
async API через tokio broadcast. Все тесты проходят, clippy/fmt/release OK.

## Deviations from design.md

1. **Feature `wasapi` cpal:** в design.md §1 и корневом Cargo.toml указано
   `features = ["wasapi"]`, но cpal 0.15 не имеет такой feature (WASAPI — дефолтный
   backend на Windows). Убрана из workspace Cargo.toml.

2. **Каналы в loopback:** дизайн использует `channels: 1` в StreamConfig. WASAPI
   loopback отклоняет mono (`AUDCLNT_E_UNSUPPORTED_FORMAT`, 0x88890008). Используется
   родное число каналов устройства, сведение в mono выполняется в `to_mono()`.

3. **AudioError варианты:** дизайн использует `cpal::DeviceUnavailable`, которого нет
   в cpal 0.15. Добавлены `HostUnavailable`, `DefaultStreamConfig`, `Resampler`.

4. **rubato 0.14 API:** `input_buffer_allocate(false)` возвращает пустые буферы
   (паника index out of bounds) — используется `true` (заполнение нулями).
   `process_into_buffer` возвращает `Result<(usize, usize)>`; выход обрезается до
   `frames_out`. Добавлен внутренний `pending` буфер для накопления до полного чанка
   (дизайн-версия копировала по одному семплу и паниковала на пустых буферах).

5. **AudioResampler::process** возвращает `Result<Vec<f32>, AudioError>` (дизайн: `Vec<f32>`)
   для проброса ошибок rubato. Добавлен `needed_input_frames()` для тестов.

6. **Хранение стримов:** дизайн использует `Arc<Mutex<Vec<Stream>>>`, но cpal `Stream`
   не `Send + Sync`, что clippy отклоняет. Используется обычный `Vec<Stream>`;
   `start_*_capture()` принимают `&mut self` (дизайн: `&self`).

7. **Пример:** design.md §6 создаёт `AudioEngine` без `mut`, но API требует `&mut self` —
   пример обновлён соответственно. Добавлены `anyhow`/`tracing-subscriber` в dev-deps.

## Verified

- `cargo build -p engine-audio`, `--examples`, `--release` — OK
- `cargo test -p engine-audio` — 6 passed (включая captures_system_audio, captures_microphone,
  dual_lane_capture, resamples_to_16khz, async_stream)
- `cargo clippy -p engine-audio --all-targets -- -D warnings` — 0 warnings
- `cargo fmt -p engine-audio -- --check` — чисто
- `cargo build --workspace` — OK