# Delta: Speech-to-Text

## ADDED Requirements

### Requirement: Groq API Client
Система SHALL отправлять аудио сегменты в Groq API для транскрипции.

#### Scenario: Успешная транскрипция (test: transcribes_successfully)
- GIVEN валидный аудио сегмент и валидный API key
- WHEN вызван `SttClient::transcribe(audio)`
- THEN возвращается `Ok(Transcript { text: "..." })`
- AND latency < 2 секунды для сегмента ≤ 7с

#### Scenario: Invalid API key (test: errors_on_invalid_key)
- GIVEN неверный API key
- WHEN вызван `SttClient::transcribe(audio)`
- THEN возвращается `Err(SttError::Authentication)`

### Requirement: Queue Management
Система SHALL обрабатывать аудио сегменты через очередь с ограниченной конкурентностью.

#### Scenario: Bounded concurrency (test: respects_concurrency_limit)
- GIVEN очередь с max_concurrency = 3
- WHEN 10 сегментов добавлены в очередь
- THEN одновременно обрабатывается ≤ 3 запроса
- AND остальные ждут в очереди

#### Scenario: Queue overflow protection (test: rejects_on_overflow)
- GIVEN очередь заполнена (100 pending requests)
- WHEN добавлен новый сегмент
- THEN возвращается `Err(SttError::QueueFull)`

### Requirement: Retry Logic
Система SHALL автоматически повторять неудачные запросы с exponential backoff.

#### Scenario: Transient error retry (test: retries_on_transient_error)
- GIVEN сегмент и временная ошибка сети (timeout)
- WHEN первый запрос падает
- THEN система повторяет запрос через 100ms
- AND второй запрос успешен
- AND возвращается `Ok(Transcript)`

#### Scenario: Max retries exceeded (test: fails_after_max_retries)
- GIVEN сегмент и постоянная ошибка (500 error)
- WHEN 3 попытки неудачны
- THEN возвращается `Err(SttError::MaxRetriesExceeded)`

### Requirement: Circuit Breaker
Система SHALL использовать circuit breaker для защиты от каскадных сбоев.

#### Scenario: Circuit opens (test: opens_circuit_on_failures)
- GIVEN 5 последовательных неудачных запросов
- WHEN обнаружено
- THEN circuit breaker переходит в Open state
- AND следующие запросы сразу возвращают `Err(SttError::CircuitOpen)` без HTTP call

#### Scenario: Circuit closes (test: closes_circuit_on_success)
- GIVEN circuit breaker в Open state
- WHEN прошел timeout (30 секунд)
- THEN circuit breaker переходит в Half-Open state
- AND следующий запрос выполняется
- AND если успешен, circuit переходит в Closed state

### Requirement: Streaming API
Система SHALL предоставлять async API для потоковой обработки сегментов.

#### Scenario: Stream processing (test: streams_transcripts)
- GIVEN поток аудио сегментов
- WHEN сегменты отправлены в `SttProcessor::process_stream()`
- THEN транскрипты возвращаются в том же порядке
- AND backpressure не блокирует producer

## MODIFIED Requirements

(none)

## REMOVED Requirements

(none)
