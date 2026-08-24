# Delta: Store

## ADDED Requirements

### Requirement: Session Persistence
Система SHALL сохранять сессию, turns и ответы в локальный sqlite.

#### Scenario: Сессия и turns записаны (test: creates_session_and_turns)
- GIVEN SessionStore с временной БД
- WHEN start_session + insert_turn x2 + end_session
- THEN в БД 1 session и 2 turns с корректными speaker/text

#### Scenario: Метрики ответа (test: records_answer_metrics)
- GIVEN insert_answer(outcome="answered", stt_latency_ms=300, ttft_ms=700)
- WHEN прочитано stats
- THEN p50/p95 считаются и возвращаются

### Requirement: Replay Log
Система SHALL писать append-only events.jsonl и wav-файлы сегментов в директорию сессии.

#### Scenario: Roundtrip лога (test: replay_log_roundtrip)
- GIVEN ReplayLogger с временной директорией
- WHEN записаны segment+transcript+turn+trigger+answer события и wav
- THEN read_events() возвращает те же события в том же порядке
- AND wav-файл существует и читается

### Requirement: Offline Replay
Система SHALL пересимулировать assembler и триггеры из replay-лога с переопределёнными порогами.

#### Scenario: Симуляция триггеров (test: replay_simulates_triggers)
- GIVEN лог с 3 turns интервьюера (короткая, длинная без "?", длинная с "?")
- WHEN replay с min_words=4
- THEN короткая не триггерит, длинные триггерят (auto и speculative)

#### Scenario: Дебаунс-отмена в симуляции (test: replay_debounce_cancel)
- GIVEN два turn интервьюера с интервалом 300мс и debounce 600мс
- WHEN replay
- THEN ровно 1 триггер (первый отменён вторым)

## MODIFIED Requirements

(none в домене store)

## REMOVED Requirements
(none)
