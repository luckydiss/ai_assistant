# Proposal: Speech-to-Text with Groq

## Why
Нужно преобразовывать аудио сегменты в текст используя Groq API (whisper-large-v3-turbo).

## What Changes
- Groq API client для STT
- Async queue с ограниченной конкурентностью
- Retries с exponential backoff
- Circuit breaker для защиты от сбоев
- Streaming API для обработки сегментов

## Scope
- HTTP client для Groq API
- Audio encoding (WAV format)
- Queue management с bounded concurrency
- Error handling и retries
- Metrics для мониторинга

## Non-Goals
- Другие STT провайдеры (только Groq в MVP)
- Local STT inference
- Multi-language support (auto-detect only)

## Affected Specs
- stt
