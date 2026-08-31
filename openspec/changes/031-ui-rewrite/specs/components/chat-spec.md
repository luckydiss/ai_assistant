# Delta: Chat Component (MODIFIED)

## MODIFIED Requirements

### Requirement: Chat Message Rendering
Chat SHALL рендерить сообщения инкрементально, виртуализировать список, мемоизировать markdown.

#### Scenario: Incremental append (test: chat_appends_incrementally)
- GIVEN chat содержит 50 сообщений
- WHEN новое сообщение добавляется (`answer_done` event)
- THEN только новое сообщение парсится и рендерится
- AND существующие 50 items НЕ re-render
- AND paint time ≤ 10ms (vs 340ms в старом UI)

#### Scenario: Виртуализация для 100+ сообщений (test: chat_virtualizes_long_history)
- GIVEN chat содержит 120 сообщений
- WHEN пользователь скроллит
- THEN в DOM рендерятся только видимые ~10 сообщений
- AND scroll плавный 60fps без jank
- AND memory usage ≤ 20MB (vs 60MB без виртуализации)

#### Scenario: Markdown memoization (test: chat_memoizes_markdown)
- GIVEN сообщение "# Title\n\nSome **bold** text"
- WHEN компонент Message рендерится первый раз
- THEN markdown парсится в HTML
- AND кэшируется в WeakMap по content hash
- WHEN тот же content рендерится снова (re-mount) → HTML берётся из кэша без парсинга

#### Scenario: Code block syntax highlighting (test: chat_highlights_code)
- GIVEN сообщение содержит \`\`\`rust\nfn main() {}\`\`\`
- WHEN рендерится
- THEN компонент CodeBlock применяет highlight.js с языком "rust"
- AND кнопка "Copy" появляется при hover
- AND клик копирует код в clipboard

### Requirement: Streaming Answer Display
Chat SHALL показывать частичное сообщение во время streaming с cursor animation.

#### Scenario: Streaming tokens (test: chat_streams_tokens)
- GIVEN Rust emit `answer_token` events: "H", "e", "l", "l", "o"
- WHEN events обрабатываются
- THEN UI показывает accumulating text "H" → "He" → "Hel" → "Hell" → "Hello"
- AND cursor/spinner анимируется в конце текста
- AND autoscroll следует за новым текстом (если пользователь внизу)

#### Scenario: Stream complete (test: chat_finalizes_stream)
- GIVEN streaming сообщение завершено (`answer_done`)
- WHEN event обработан
- THEN cursor исчезает
- AND сообщение перемещается из `partialMessage` в `messages[]`
- AND markdown парсится окончательно

### Requirement: Autoscroll Behavior
Chat SHALL автоматически скроллить к новым сообщениям, но отключаться если пользователь скроллит вверх.

#### Scenario: Autoscroll enabled (test: chat_autoscrolls_bottom)
- GIVEN пользователь внизу чата (scrollTop + clientHeight ≥ scrollHeight - 50px)
- WHEN новое сообщение добавляется
- THEN чат автоматически скроллит к низу (smooth behavior)

#### Scenario: Autoscroll disabled (test: chat_preserves_scroll_position)
- GIVEN пользователь прокрутил вверх (читает старое сообщение)
- WHEN новое сообщение добавляется
- THEN scroll position НЕ меняется
- AND badge "↓ New message" появляется внизу экрана

#### Scenario: Re-enable autoscroll (test: chat_reenables_autoscroll)
- GIVEN autoscroll был отключён (пользователь наверху)
- WHEN пользователь скроллит вниз до конца
- THEN autoscroll re-enable автоматически
- AND следующее сообщение триггерит autoscroll

### Requirement: Message Grouping
Chat SHALL группировать последовательные сообщения одного спикера.

#### Scenario: Group consecutive turns (test: chat_groups_same_speaker)
- GIVEN сообщения: [User:"A", User:"B", Assistant:"C", Assistant:"D"]
- WHEN рендерятся
- THEN отображаются 2 группы:
  - Group 1 (User): ["A", "B"] — аватар один, timestamp один
  - Group 2 (Assistant): ["C", "D"]

#### Scenario: New speaker breaks group (test: chat_breaks_group_on_speaker_change)
- GIVEN сообщения: [User:"A", Assistant:"B", User:"C"]
- WHEN рендерятся
- THEN отображаются 3 группы (каждое сообщение отдельно)

### Requirement: Collapse Transcripts/Operations (from config)
Chat SHALL сворачивать системные сообщения по настройкам `config.chat.collapse_*`.

#### Scenario: Collapse transcripts (test: chat_collapses_transcripts)
- GIVEN `config.chat.collapse_transcripts = true`
- WHEN сообщение типа "transcript" (STT result) рендерится
- THEN показывается collapsed: "🎤 Transcript" (expandable)
- AND клик разворачивает полный текст

#### Scenario: Collapse operations (test: chat_collapses_operations)
- GIVEN `config.chat.collapse_operations = true`
- WHEN сообщение типа "context_updated" рендерится
- THEN показывается collapsed: "⚙️ Context updated" (expandable)

### Requirement: Performance Target
Chat render SHALL выполняться в пределах performance budget.

#### Scenario: Initial render 50 messages (test: perf_initial_render)
- GIVEN пустой chat
- WHEN загружается история из 50 сообщений
- THEN initial paint ≤ 120ms (измерено Chrome DevTools Performance)

#### Scenario: Append 1 message (test: perf_append_one)
- GIVEN chat с 50 сообщениями
- WHEN добавляется 1 новое сообщение
- THEN paint time ≤ 10ms

## ADDED Requirements
(см. Виртуализация, Markdown memoization выше)

## REMOVED Requirements

### REMOVED: Full rerender on every message
Старый UI вызывал `render_chat()` с полной перерисовкой всех сообщений — удалено в пользу инкрементального.

### REMOVED: innerHTML-based rendering
Старый UI генерировал HTML строки и присваивал через `innerHTML` — заменено на компонентный подход с реактивностью.
