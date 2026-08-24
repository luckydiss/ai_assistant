# Delta: LLM Client

## ADDED Requirements

### Requirement: SSE Streaming
Система SHALL парсить SSE-поток OpenAI-compatible endpoint и отдавать дельты текста как события.

#### Scenario: Парсинг data-строки (test: parses_sse_data_lines)
- GIVEN строка "data: {json}"
- WHEN вызван parse_sse_line
- THEN возвращён payload без префикса

#### Scenario: Извлечение дельты (test: extracts_delta)
- GIVEN json с choices[0].delta.content = "Пр"
- WHEN вызван extract_delta
- THEN возвращено Some("Пр")

#### Scenario: Стрим с mock-сервера (test: streams_tokens_from_mock_server)
- GIVEN локальный mock SSE-сервер с 2 дельтами и [DONE]
- WHEN вызван stream_answer
- THEN получены Token("Пр"), Token("ивет"), Done("Привет")

### Requirement: SKIP Protocol
Система SHALL буферизовать первые токены и отменять вывод при "<SKIP>".

#### Scenario: SKIP обнаружен (test: skip_detected)
- GIVEN дельты "<", "SKI", "P>"
- WHEN скормлены в feed_skip последовательно
- THEN первые два раза Buffering, затем Skipped

#### Scenario: Обычный ответ (test: passthrough_after_partial)
- GIVEN дельта "Привет"
- WHEN feed_skip
- THEN Passthrough, буфер содержит "Привет"

#### Scenario: SKIP не показывает токены (test: skip_emits_no_tokens)
- GIVEN mock-сервер отвечает "<SKIP>"
- WHEN stream_answer
- THEN получено только событие Skipped, Token-событий нет

### Requirement: Cancellation
Система SHALL поддерживать отмену активного стрима.

#### Scenario: Abort останавливает стрим (test: cancel_aborts_stream)
- GIVEN медленный mock-сервер (задержка 500мс до тела)
- WHEN вызван stream_answer и сразу abort()
- THEN receiver закрывается без событий Done/Skipped

### Requirement: Error Surfacing
Система SHALL сообщать ошибки как AnswerEvent::Failed без паники.

#### Scenario: 401 (test: fails_on_401)
- GIVEN mock-сервер отвечает HTTP 401
- WHEN stream_answer
- THEN получено Failed с упоминанием auth

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
