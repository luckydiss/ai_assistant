# Delta: LLM (endpoint search tool)

## ADDED Requirements

### Requirement: Server-Side Search Tool
Если [llm] search_enabled=true, запрос SHALL содержать инжект [llm] search_tool_json (raw JSON, merge в тело запроса). Если false — тело без изменений.

#### Scenario: Поиск включён (test: search_tool_injected)
- GIVEN search_enabled=true, search_tool_json='{"enable_search":true}'
- WHEN запрос на mock с захватом тела
- THEN тело содержит "enable_search":true

#### Scenario: Выключен (test: search_tool_absent)
- GIVEN search_enabled=false
- THEN тело не содержит enable_search

## MODIFIED / REMOVED: (none)
