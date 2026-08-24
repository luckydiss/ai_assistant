# Tasks: Overlay 1:1

- [x] 1.1 Config-секции по design.md §1 + тест validates_opacity
  verify: `cargo test -p engine-config`

- [x] 1.2 Store: chats/notes таблицы + методы, тесты chats_roundtrip, notes_roundtrip
  verify: `cargo test -p engine-store`

- [x] 2.1 Orchestrator multi-chat + set_active_chat/reset_active + Cmd::SetAuto; тесты chats_isolated, active_chat_gets_turns, reset_active_clears
  verify: `cargo test -p engine-orchestrator`

- [x] 2.2 Search injection в engine-llm; тесты search_tool_injected, search_tool_absent
  verify: `cargo test -p engine-llm`

- [x] 2.3 STT language в multipart; тест stt_language_sent
  verify: `cargo test -p engine-llm`

- [x] 3.1 Команды desktop по design.md §9
  verify: `cargo build -p desktop`

- [x] 3.2 Индикатор-окно по design.md §8 + stealth на него
  verify: manual — бейджи в углу, не попадают в захват

- [x] 4.1 overlay.html + CSS по design.md §6
  verify: manual — layout совпадает с зоной-схемой, прозрачность из конфига

- [x] 4.2 overlay.js по карте §7 (все обработчики)
  verify: manual — каждый пункт карты проверен кликом

- [ ] 5.1 Боевая проверка: 2 чата с разными контекстами; переключение; сброс; тумблеры авто/озвучка; заметки-просмотр
  verify: manual чек-лист

- [x] 6.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0

## STOP Protocol
Если элемент карты §7 не имеет соответствующей команды — НЕ выдумывать поведение; вопрос человеку.
CSS/HTML правки без изменения id из §6 — допустимы; новые id — только через вопрос.
