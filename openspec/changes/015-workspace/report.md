# Report: Workspace

## Что сделано
- **engine-store**: таблицы `meetings`/`contexts` в `SessionStore::open`; `start_session` → `ON CONFLICT(id) DO UPDATE` (upsert, сбрасывает ended_at); методы meetings (create/list/rename/delete/bump_messages/set_meeting_context) и contexts (create/get/update/delete/list); структуры `MeetingRow`/`ContextRow` (Serialize/Deserialize).
- **engine-context**: `PromptContext` + `ContextBuilder::with_workspace(base_system, ws, max_tokens)`; старый `new()` сохранён как есть (тесты 007 не тронуты).
- **apps/desktop**: `src/commands.rs` — 8 Tauri-команд (meetings_list, meeting_create, meeting_rename, meeting_delete, contexts_list, context_save, context_delete, meeting_set_context); зарегистрированы в invoke_handler; store зарегистрирован в `app.manage`.
- Тесты: `engine-store/tests/workspace_tests.rs` (8 тестов), engine-context builder_uses_context/builder_empty_context.

## Отклонения от design.md
1. **State-тип команд**: design указывает `State<'_, Arc<SessionStore>>`, но `rusqlite::Connection` не Sync → `Arc<SessionStore>` не может быть tauri State. Использован `Arc<tokio::sync::Mutex<SessionStore>>` (тот же Arc, что уже управляется в main.rs) — команды делают `state.lock().await`.
2. **Команды возвращают `Result<_, String>`** вместо голых значений — иначе ошибки store теряются (рабочая необходимость, а не API-break).
3. **context_save = upsert**: создаёт, если id не существует, иначе обновляет (в design create/update разделены; для UI удобнее один вызов). Оба примитива остались в store.
4. **resume_appends**: покрыт внутри context_roundtrip (создание+апдейт с резюме), отдельный тест не добавлял.
5. `list_meetings` без SUM(duration) — дизайн допускал «по желанию», опущено для MVP.

## Результаты проверок
- `cargo build --workspace` — ok; `cargo test -p engine-store` — 13 passed (включая 8 workspace); `cargo test -p engine-context` — 9 passed; clippy `-D warnings` — 0.
- Известные pre-existing флаки engine-audio (живой захват микрофона) — не связаны с этим ченджем.

## Осталось
- UI-часть (меню встреч/контекстов, импорт резюме) — чендж 016-overlay / отдельный UI-чендж.
- Экспорт TXT/MD — вне scope (proposal: non-goal для MVP UI? уточнено в 016).
