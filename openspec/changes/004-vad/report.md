# Report: Voice Activity Detection (004)

## Summary

Реализован `crates/engine-vad`: Silero VAD через ort (ONNX Runtime), сегментация речи по паузам тишины (600ms) и лимиту 7с, async API через tokio mpsc. 8/8 тестов проходят, clippy/fmt/release OK. Example `vad_demo` показывает корректную сегментацию реального аудио (aepyx.wav).

## Deviations from design.md

1. **Модель v5 вместо v4 (вход 576 вместо 512):** дизайн написан под Silero v4 (вход [1,512]). Актуальная модель в master `snakers4/silero-vad` — v5, требует вход **576 samples = 512 + 64 context**. Реализация хранит внутренний `context`-буфер (последние 64 семпла) и подаёт на вход 576. Это единственная причина, почему детектирование изначально давало ~0.0005 — модель v5 без контекста не видит речь.

2. **ort 2.0 API вместо 1.x:** дизайн использует ort 1.x синтаксис (`Session::builder()?`, ndarray-тензоры, `outputs["output"]`). project.md фиксирует ort 2.0. Фактически: `Session::builder()` возвращает `Result`, все builder-методы `.map_err()`; `Tensor::from_array((shape, vec))` вместо ndarray; `outputs.get("output")`; `try_extract_tensor::<f32>()` возвращает `(&Shape, &[f32])`, не ndarray. ort 2.0 стабильной версии нет — в workspace указан pre-release `2.0.0-rc.13`.

3. **`sr` вход — Int64 scalar ([], не [1]):** дизайн передаёт `sr` как [1]. Оба варианта работают одинаково у этой модели (проверено), оставлен `[1]` с `vec![16000i64]`.

4. **onnxruntime.dll (load-dynamic):** ort 2.0 с feature `load-dynamic` ищет `onnxruntime.dll`. На машине в System32 была старая версия 1.17 → panic `BadVersion`. Скачан ORT 1.28.0 (`onnxruntime-win-x64-1.28.0.zip`) и положен в корень проекта. Тесты задают `ORT_DYLIB_PATH` явно. В report 004 добавлена runtime-зависимость.

5. **Модель/файлы загрузки:** `Invoke-WebRequest` и `raw.githubusercontent.com` падали (TLS, rate limit). Файл скачан через GitHub API `Accept: application/vnd.github.raw`. Вместо синтетического аудио (дизайн §8.4) используется реальный речевой `aepyx.wav` из репозитория silero-vad — синтетика не проходит порог 0.5.

6. **Тест `preserves_context` вместо `merges_short_pause`:** spec называет сценарий `preserves_context` (речь, пауза 200ms, речь → один сегмент). Реализован под этим именем.

7. **Лимит max_segment_ms не превышается:** сегментер эмитит ДО добавления чанка, если `segment_duration_ms + 32 > max_segment_ms` (в дизайне проверка `>=` после добавления давала превышение на 32ms). Гарантирует `duration_ms <= max_segment_ms`.

8. **`ndarray = "0.15"` не используется:** дизайн пинит ndarray 0.15, ort 2.0 тянет ndarray 0.17. Зависимость оставлена в Cargo.toml, но в коде не используется (тензоры через `Tensor::from_array`).

## Model Files (корень проекта)

- `silero_vad.onnx` — модель (~2.3MB, MIT), версия v5.
- `onnxruntime.dll` — ONNX Runtime 1.28.0 (x64) для runtime.
- `aepyx.wav` — референсное речевое аудио для тестов.

## Verified

- `cargo test -p engine-vad` — 8/8 ok
- `cargo clippy -p engine-vad --all-targets -- -D warnings` — ok
- `cargo build -p engine-vad --release` — ok
- `cargo build --workspace` — ok
- `cargo run -p engine-vad --example vad_demo` — сегменты: 6976ms/6976ms/6976ms/4288ms/5824ms/... (≤7000ms)
