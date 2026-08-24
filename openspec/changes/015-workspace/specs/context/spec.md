# Delta: Context (workspace integration)

## MODIFIED Requirements

### Requirement: Message Assembly
Система SHALL собирать запрос из system-промпта, обогащённого активным контекстом (role → persona; extra_prompt + резюме + вакансия → system), и user-блока (summary + диалог + фокус).

#### Scenario: Контекст в промпте (test: builder_uses_context)
- GIVEN контекст role="Rust dev", extra="решай как человек", resume="5 лет", vacancy="Ozon"
- WHEN build()
- THEN system содержит role и extra; user-блок содержит резюме и вакансию

#### Scenario: Пустой контекст (test: builder_empty_context)
- GIVEN контекст без полей
- WHEN build()
- THEN промпт равен базовому system из конфига

## ADDED Requirements
(none)

## REMOVED Requirements
(none)
