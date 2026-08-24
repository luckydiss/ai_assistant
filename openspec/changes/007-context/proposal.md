# Proposal: Context Builder

## Why
Нужно собирать промпт для LLM из persona, summary, недавних реплик и фокус-вопроса с контролем токен-бюджета.

## What Changes
- ContextBuilder: сборка messages[] для OpenAI-compatible endpoint
- Оценка токенов без внешних библиотек (chars/4)
- Усечение старейших реплик при переполнении бюджета

## Scope
- Типы ChatMessage/Role (общие с engine-llm)
- build(summary, turns, focus, note) -> Vec<ChatMessage>
- Token budget truncation

## Non-Goals
- tiktoken (нет интернета для BPE-файлов)
- Multimodal (скриншоты) — позже

## Affected Specs
- context
