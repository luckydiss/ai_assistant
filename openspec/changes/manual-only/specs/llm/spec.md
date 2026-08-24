# Delta: LLM (SKIP removed)

## REMOVED Requirements

### Requirement: SKIP Protocol
(Удалено целиком: skip.rs, feed_skip, SkipState, AnswerEvent::Skipped. Все запросы явные, гейт не нужен.)

## MODIFIED Requirements

### Requirement: SSE Streaming
Клиент SHALL начинать стриминг токенов сразу (без буферизации SKIP) и передавать reasoning_effort в тело запроса, если задан.

#### Scenario: Стрим с mock-сервера (прежний streams_tokens_from_mock_server сохраняется)
#### Scenario: reasoning_effort в теле (test: reasoning_effort_sent)
- GIVEN LlmClient с reasoning_effort=Some("low")
- WHEN запрос на mock с захватом тела
- THEN тело содержит "reasoning_effort":"low"

#### Scenario: Отмена (прежний cancel_aborts_stream сохраняется)
#### Scenario: 401 (прежний fails_on_401 сохраняется)

## ADDED Requirements
(none)
