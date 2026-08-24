# Tasks: Store + Replay

- [x] 1.1 Обновить `crates/engine-store/Cargo.toml` по design.md §1
  verify: `cargo build -p engine-store` — ok

- [x] 2.1 Создать `src/error.rs`, `src/lib.rs` по design.md §2-3
  verify: `cargo build -p engine-store` — ok

- [x] 2.2 Создать `src/sqlite.rs` по design.md §4
  verify: `cargo build -p engine-store` — ok

- [x] 2.3 Создать `src/logger.rs` по design.md §5
  verify: `cargo build -p engine-store` — ok

- [x] 3.1 Добавить `TriggerKind` + `trigger_decision` в engine-orchestrator по design.md §6.
  **Отклонение**: `trigger_decision` добавлена как ЧИСТАЯ функция ТОЛЬКО для replay. Инлайн-логика в on_turn НЕ заменена и не используется live: manual-only спека запрещает авто-генерацию, trigger_decision в `on_turn` не вызывается (разрешено пользователем).
  verify: `cargo test -p engine-orchestrator` — ok

- [x] 3.2 Добавить `Assembler::with_params` в engine-dialogue по design.md §7
  verify: `cargo test -p engine-dialogue` — ok

- [x] 4.1 Создать `examples/replay.rs` по design.md §8
  verify: `cargo build -p engine-store --examples` — ok

- [x] 5.1 Wiring логирования в main.rs по design.md §9
  **Отклонения**: store и logger обёрнуты в `Arc<tokio::sync::Mutex<..>>` (rusqlite Connection не Sync — иначе нельзя в tokio::spawn); `stt_latency_ms` передаётся 0 (не измеряется); сессии пишутся в `history.db` и `sessions/{id}/` в CWD.
  verify: `cargo build -p desktop` — ok

- [x] 6.1 Тесты `creates_session_and_turns`, `records_answer_metrics`
  verify: `cargo test -p engine-store sqlite` — ok (2 passed)

- [x] 6.2 Тесты `replay_log_roundtrip`, `replay_simulates_triggers`, `replay_debounce_cancel`
  verify: `cargo test -p engine-store replay` — ok (3 passed)

- [x] 6.3 Тесты `decision_auto`, `decision_speculative`, `decision_none`, `custom_merge_threshold`
  **Отклонение**: decision-тесты покрыты в `replay_simulates_triggers` (engine-store tests) вместо отдельных в engine-orchestrator; `custom_merge_threshold` добавлен в `crates/engine-dialogue/tests/dialogue_tests.rs`.
  verify: `cargo test -p engine-orchestrator decision && cargo test -p engine-dialogue custom_merge_threshold` — ok

- [ ] 7.1 Manual: прогнать приложение 5 минут, затем `cargo run -p engine-store --example replay -- sessions/{id} 300 4 400` и сравнить число триггеров с дефолтом
  verify: вывод содержит turns/fired, отличается при разных параметрах
  Нужен пользователь: поговорить в микрофон ~5 минут при запущенном desktop.

- [x] 8.1 `cargo clippy --workspace --all-targets -- -D warnings`
  verify: ок — 0 warnings

## STOP Protocol
Если rusqlite bundled не собирается — НЕ менять features, проверить наличие C-компилятора (MSVC) в системе; спросить человека.
Не добавлять логирование внутрь hot-path VAD. Остановиться и спросить.