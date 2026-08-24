# Delta: Context (no SKIP protocol)

## MODIFIED Requirements

### Requirement: Message Assembly
Система SHALL собирать запрос из system-промпта (без инструкции <SKIP>) и user-блока (summary + диалог + опциональный фокус + note). User-блок SHALL завершаться строкой «Ответь по запросу кандидата.».

#### Scenario: Полный контекст (test: builds_full_context)
- GIVEN persona, summary "S", 2 turns, focus Some
- WHEN build()
- THEN messages[0].role=system содержит persona; messages[1] содержит "S", реплики, фокус

#### Scenario: Без фокуса (test: builds_without_focus)
- GIVEN нет реплик I, note Some("помоги")
- WHEN build()
- THEN user-блок содержит note и НЕ содержит строку «Последний вопрос I»

#### Scenario: Без SKIP-инструкции (test: no_skip_instruction)
- GIVEN любой вход
- WHEN build()
- THEN ни одно сообщение не содержит "<SKIP>"

### Requirement: Token Budget
(Без изменений: truncates_oldest_turns, keeps_short_dialogue сохраняются; вызовы build обновить под новую сигнатуру.)

### Requirement: Manual Note
(Без изменений: appends_note сохраняется.)

## REMOVED Requirements
(Сценарий includes_skip_protocol удалён вместе с протоколом.)
