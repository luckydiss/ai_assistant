# Proposal: Manual-Only Assistant (удаление авто-генерации)

## Why
Авто-ответы по репликам интервьюера нестабильны (ложные SKIP, недетерминизм модели) и ломают UX. Продуктовое решение: ассистент отвечает ТОЛЬКО по явному запросу — кнопка «Что сказать» / хоткей, ручной ввод. Транскрипт диалога продолжает копиться как контекст для запроса.

## What Changes
- Удалить авто- и спекулятивные триггеры из orchestrator (on_turn только пополняет контекст)
- Удалить SKIP-протокол целиком: skip.rs, SkipState, AnswerEvent::Skipped, упоминания в промптах
- System-промпт без инструкции <SKIP>
- Config: убрать [orchestrator] (debounce/min_words), добавить llm.reasoning_effort
- UI: убрать обработку skipped; статус listening с подсказкой хоткея

## Scope
- Правки engine-orchestrator, engine-llm, engine-context, engine-config, desktop wiring, overlay.js
- Обновление тестов: удалить сценарии авто/SKIP, добавить сценарии manual-only

## Non-Goals
- Возврат авто-режима (если понадобится — отдельный change с новой спекой)
- Обработка reasoning_content (не нужна без SKIP-гейта; reasoning_effort=low достаточно)

## Affected Specs
- orchestrator (REMOVED/MODIFIED), llm (REMOVED), context (MODIFIED), config (MODIFIED), ui (MODIFIED)
