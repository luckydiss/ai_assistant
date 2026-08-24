# Tasks: Screenshots + Vision

- [x] 1.1 Добавить png и base64 в workspace; windows += Win32_Graphics_Gdi
  verify: `cargo build -p desktop` — ok

- [x] 2.1 capture.rs по design.md §2 (GDI BitBlt + crop + encode_png)
  verify: `cargo build -p desktop` — ok

- [x] 2.2 Тест crop_window_region (чистый crop)
  verify: `cargo test -p desktop` — ok (3 unit-теста crop в capture.rs, без Windows-API)
  Решение по STOP-подсказке: crop-тесты оставлены unit-тестами в bin `desktop` (cargo test -p desktop их поднимает); отдельный крейт не нужен

- [x] 2.3 Тест capture_produces_png (manual-run на Windows)
  verify: `cargo run -p desktop --example shot` пишет shot.png — ok (2048x1152, 2.2 MB)

- [x] 3.1 MessageContent/Part в engine-context; build(..., image_b64) по design.md §3
  verify: `cargo test -p engine-context` — ok (10 passed, включая vision_payload_contains_image_and_text)

- [x] 3.2 manual(note, image) в orchestrator по design.md §4
  verify: `cargo test -p engine-orchestrator` — ok (8 passed)

- [x] 4.1 screen_analyze + dispatch хоткеев по design.md §5
  verify: `cargo build -p desktop` — ok

- [x] 4.2 Тест vision_payload_sent с mock-сервером захвата тела по design.md §6
  verify: `cargo test -p engine-orchestrator vision_payload_sent` — ok (тело содержит image_url и data:image/png;base64,QUJD)

- [x] 5.1 Кнопка «Анализ экрана» в overlay.js
  verify: manual — ответ учитывает экран (кнопка + hotkey Ctrl+H/Ctrl+Shift+H; боевая проверка ждёт пользователя)

- [x] 6.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0 — ok (кроме известных флаков engine-audio)

## STOP Protocol
BITMAPINFO::default() компилируется в windows 0.52 — замена на zeroed() не понадобилась.