# Delta: State Management (ADDED)

## ADDED Requirements

### Requirement: Config Store Synchronization
`config` store SHALL синхронизироваться с Rust `Config` через Tauri events bidirectionally.

#### Scenario: Initial load (test: config_store_loads_initial)
- GIVEN Rust config содержит `llm.provider = "openrouter"`
- WHEN UI инициализируется (`initConfig()`)
- THEN `$config.llm.provider === "openrouter"` в течение 100ms

#### Scenario: Config change event (test: config_store_reacts_to_rust_change)
- GIVEN Rust watcher emit `config_changed` event (файл изменён извне)
- WHEN event получен UI
- THEN `config` store обновляется реактивно
- AND все компоненты, использующие `$config`, re-render

#### Scenario: UI → Rust update (test: config_store_updates_rust)
- GIVEN UI вызывает `updateConfig('llm.provider', 'dslab')`
- WHEN команда выполнена
- THEN Rust сохраняет config.toml
- AND emit `config_changed` → store получает подтверждение

#### Scenario: Optimistic updates (test: config_store_optimistic)
- GIVEN UI вызывает `updateConfig('ui.opacity', 90)`
- WHEN команда в процессе (200ms latency)
- THEN store немедленно обновляется локально (optimistic)
- AND если Rust возвращает ошибку → rollback к предыдущему значению

### Requirement: Chat Store Event Stream
`chat` store SHALL накапливать сообщения из Rust events (`dialogue_turn`, `answer_token`, `answer_done`).

#### Scenario: Turn completed (test: chat_store_turn)
- GIVEN Rust emit `dialogue_turn` с Turn {speaker: "User", content: "Hello"}
- WHEN event получен
- THEN `$chat.messages` содержит новый turn
- AND компоненты MessageList re-render инкрементально (только новый item)

#### Scenario: Streaming answer (test: chat_store_streaming)
- GIVEN Rust emit последовательность `answer_token` ("H", "e", "l", "l", "o")
- WHEN events получены
- THEN `$chat.partialMessage` accumulates "Hello"
- AND `$chat.streaming = true`
- AND UI показывает cursor/spinner

#### Scenario: Stream complete (test: chat_store_stream_done)
- GIVEN streaming answer завершён (`answer_done`)
- WHEN event получен
- THEN `$chat.partialMessage` перемещается в `messages[]`
- AND `$chat.streaming = false`

### Requirement: Models Store Lazy Loading
`models` store SHALL загружать каталог моделей при первом обращении, кэшировать на 5 минут.

#### Scenario: First access (test: models_store_lazy_load)
- GIVEN store не инициализирован
- WHEN компонент вызывает `await loadModels()`
- THEN `invoke('models_list')` выполняется
- AND результат сохраняется в `$models.list`

#### Scenario: Cache hit (test: models_store_cache)
- GIVEN models загружены 2 минуты назад
- WHEN другой компонент вызывает `await loadModels()`
- THEN `invoke` НЕ вызывается, возвращается кэшированный результат

#### Scenario: Cache expiry (test: models_store_cache_expire)
- GIVEN models загружены 6 минут назад
- WHEN компонент вызывает `await loadModels()`
- THEN `invoke('models_list')` выполняется заново

### Requirement: UI Ephemeral State
`ui` store SHALL управлять состоянием модалок, toast notifications, loading indicators.

#### Scenario: Toast queue (test: ui_store_toast_queue)
- GIVEN вызваны подряд `toast.success("A")`, `toast.error("B")`, `toast.info("C")`
- WHEN toasts отображаются
- THEN показываются последовательно (max 3 одновременно)
- AND каждый автоматически скрывается через 3s (success/info) или 5s (error)

#### Scenario: Modal stack (test: ui_store_modal_stack)
- GIVEN открыты ModelModal → внутри открыт ContextModal (nested)
- WHEN пользователь нажимает Escape
- THEN закрывается только ContextModal (верхний в стеке)
- AND ModelModal остаётся открытой

### Requirement: Type-Safe Store Access
Все stores SHALL экспортировать типизированные getter/setter.

#### Scenario: Type error prevents compilation (test: store_type_check)
- GIVEN код пытается присвоить `$config.llm.provider = 123` (number вместо string)
- WHEN `npm run type-check`
- THEN ошибка компиляции: "Type 'number' is not assignable to type 'string'"

## MODIFIED Requirements
(нет)

## REMOVED Requirements
(нет)
