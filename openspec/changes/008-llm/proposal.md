# Proposal: LLM Client (OpenAI-compatible, streaming)

## Why
Нужен стриминговый клиент к любому OpenAI-compatible endpoint с SKIP-протоколом и отменой.

## What Changes
- SSE-парсинг стрима chat/completions
- SKIP-буфер: первые токены решают, показывать ли ответ
- Отмена через AbortHandle (last-trigger-wins)
- AnswerEvent-канал в оркестратор

## Scope
- LlmClient::stream_answer -> (Receiver<AnswerEvent>, AbortHandle)
- Чистые функции parse_sse_line / extract_delta / feed_skip (тестируемы офлайн)
- Mock SSE server helper для тестов (без интернета)

## Non-Goals
- Retries для стриминга (отмена и новый запрос вместо retry)
- Function calling, vision

## Affected Specs
- llm
