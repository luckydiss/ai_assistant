# Proposal: Session Store and Replay

## Why
Нужна локальная история сессий (turns, ответы, метрики латентности) и replay-лог для офлайн-тюнинга порогов без живых собеседований.

## What Changes
- engine-store: sqlite (sessions/turns/answers) + JSONL replay-лог + wav-файлы сегментов
- Wiring в main.rs: логирование событий пайплайна, метрики stt_latency/ttft
- Assembler::with_params — настраиваемые пороги (для replay)
- Чистая функция trigger_decision в orchestrator (для replay)
- examples/replay.rs: офлайн-симуляция assembler+triggers с переопределёнными параметрами

## Scope
- sqlite schema + CRUD
- ReplayLogger (events.jsonl + audio/{id}.wav)
- Replay runner (compare triggers при разных порогах)

## Non-Goals
- Replay VAD/STT (хранятся пост-VAD сегменты и готовые транскрипты)
- UI для истории (позже)
- Шифрование БД

## Affected Specs
- store, dialogue (MODIFIED), orchestrator (ADDED)
