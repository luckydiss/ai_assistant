# Tasks: Workspace

- [x] 1.1 Добавить таблицы meetings/contexts в SessionStore::open по design.md §1
  verify: `cargo test -p engine-store` (старые зелёные) — ok

- [x] 1.2 start_session → upsert по design.md §1
  verify: `cargo test -p engine-store start_session_upsert` — ok

- [x] 2.1 Реализовать методы meetings по design.md §2
  verify: `cargo test -p engine-store meeting` — ok

- [x] 2.2 Реализовать методы contexts по design.md §2
  verify: `cargo test -p engine-store context_roundtrip` — ok

- [x] 2.3 Тесты meeting_create_list, meeting_rename_delete, meeting_counters_update, resume_appends, active_context_per_meeting, import_resume_text
  verify: `cargo test -p engine-store` — ok (workspace_tests.rs: 8 passed; resume_appends покрыт в context_roundtrip)

- [x] 3.1 Добавить PromptContext + with_workspace в engine-context по design.md §3
  verify: `cargo test -p engine-context` (старые зелёные) — ok

- [x] 3.2 Тесты builder_uses_context, builder_empty_context
  verify: `cargo test -p engine-context builder` — ok

- [x] 4.1 Создать apps/desktop/src/commands.rs по design.md §4 и зарегистрировать в invoke_handler
  verify: `cargo build -p desktop` — ok

- [x] 5.1 `cargo clippy --workspace --all-targets -- -D warnings`
  verify: выход 0 — ok

## STOP Protocol
Если ON CONFLICT не поддерживается версией sqlite — НЕ понижать синтаксис наугад; bundled-русqlite даёт свежий sqlite, проверить feature "bundled". Спросить при сомнении. — bundled sqlite 3.45+, ON CONFLICT работает (тест start_session_upsert зелёный).