# Proposal: Trigger Orchestrator

## Why
Нужен мозг пайплайна: решать, когда спрашивать LLM, собирать контекст, управлять отменами.

## What Changes
- State machine: IDLE/SPEAKING/ARMED/GENERATING
- Дебаунс-триггер по паузе интервьюера + спекулятивный триггер на длинный вопрос
- last-trigger-wins отмена стрима
- Ручной триггер с note

## Scope
- Orchestrator (Arc<Mutex<Inner>> + command channel)
- OrchEvent broadcast в UI
- min_words фильтр

## Non-Goals
- Роутинг моделей по типу вопроса
- TTS

## Affected Specs
- orchestrator
