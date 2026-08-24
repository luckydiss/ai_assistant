# Delta: Orchestrator

## ADDED Requirements

### Requirement: Debounced Trigger
Система SHALL запрашивать LLM после паузы интервьюера (debounce_ms), если реплика ≥ min_words слов.

#### Scenario: Триггер после паузы (test: triggers_after_debounce)
- GIVEN debounce 100мс (тестовый конфиг) и turn от Interviewer с 10 словами
- WHEN on_turn вызван
- THEN в течение 500мс mock-сервер получил ровно 1 запрос
- AND UI получил Token-события

#### Scenario: Короткая реплика игнорируется (test: short_turn_ignored)
- GIVEN turn "ок понятно да" (3 слова) и min_words=4
- WHEN on_turn вызван
- THEN mock-сервер получил 0 запросов через 500мс

### Requirement: Speculative Trigger
Система SHALL триггерить раньше (200мс), если реплика > 14 слов и содержит "?".

#### Scenario: Длинный вопрос триггерит быстро (test: speculative_trigger_fast)
- GIVEN turn с 20 словами и "?"
- WHEN on_turn вызван
- THEN запрос уходит быстрее, чем debounce_ms

### Requirement: Last-Trigger-Wins
Система SHALL отменять незавершённый ответ при новом триггере.

#### Scenario: Новый вопрос отменяет старый ответ (test: new_trigger_cancels_previous)
- GIVEN медленный mock-сервер и первый триггер уже в GENERATING
- WHEN приходит второй turn-триггер
- THEN первый стрим прерван (Token-события первого не продолжаются)
- AND выполнен запрос по второму turn

### Requirement: Skip Silence
Система SHALL не показывать UI-события при ответе "<SKIP>".

#### Scenario: SKIP скрыт (test: skip_hidden_from_ui)
- GIVEN mock-сервер отвечает "<SKIP>"
- WHEN триггер сработал
- THEN UI не получил Token-событий

### Requirement: Manual Trigger
Система SHALL поддерживать ручной триггер из любого состояния.

#### Scenario: Ручной триггер (test: manual_trigger_fires)
- GIVEN нет turn от интервьюера
- WHEN manual(Some("помоги с кодом"))
- THEN mock-сервер получил запрос с note в теле

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
