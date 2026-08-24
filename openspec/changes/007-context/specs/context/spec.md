# Delta: Context Builder

## ADDED Requirements

### Requirement: Message Assembly
Система SHALL собирать запрос из system-промпта (с persona) и user-блока (summary + диалог + фокус-вопрос).

#### Scenario: Полный контекст (test: builds_full_context)
- GIVEN persona "Rust dev", summary "S", 2 turns, focus "Как работает async?"
- WHEN вызван build()
- THEN messages[0].role = system и содержит persona
- AND messages[1].content содержит "S", обе реплики и фокус-вопрос

#### Scenario: SKIP-протокол в system (test: includes_skip_protocol)
- GIVEN system-промпт из конфига содержит "<SKIP>"
- WHEN вызван build()
- THEN messages[0].content содержит "<SKIP>"

### Requirement: Token Budget
Система SHALL усекать старейшие реплики, если оценка токенов превышает max_tokens.

#### Scenario: Усечение старейших (test: truncates_oldest_turns)
- GIVEN max_tokens = 100 и 6 длинных turns
- WHEN вызван build()
- THEN последняя реплика и focus присутствуют
- AND старейшая реплика отсутствует

#### Scenario: Бюджет не трогает короткий диалог (test: keeps_short_dialogue)
- GIVEN max_tokens = 8000 и 2 короткие turns
- WHEN вызван build()
- THEN обе turns присутствуют

### Requirement: Manual Note
Система SHALL добавлять ручной комментарий пользователя в user-блок.

#### Scenario: Note добавлен (test: appends_note)
- GIVEN note = Some("смотри на код в IDE")
- WHEN вызван build()
- THEN messages[1].content содержит "смотри на код в IDE"

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
