# Proposal: Multi-Language Answers

## Why
Как в sobes: 1-й язык списка — генерация, 2–3-й — машинный перевод ответа, отображается в оверлее.

## What Changes
- engine-llm: неблокирующий complete() + translate(text, lang)
- orchestrator: на Done запускает ≤ 2 переводов, отменяет их при новом триггере; OrchEvent::Translation
- languages текут из активного контекста (PromptContext) в orchestrator при старте пайплайна
- overlay: блоки перевода под ответом

## Scope
- Перевод только готового ответа (не стрим)
- До 2 дополнительных языков

## Non-Goals
- Перевод транскрипта входящей речи
- Стриминговый перевод

## Affected Specs
- translations (ADDED), orchestrator (ADDED), context (MODIFIED), ui (ADDED)
