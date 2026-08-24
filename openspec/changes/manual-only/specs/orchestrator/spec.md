# Delta: Orchestrator (manual-only)

## REMOVED Requirements

### Requirement: Debounced Trigger
(Удалено: авто-генерация по паузе интервьюера отменена продуктовым решением.)

### Requirement: Speculative Trigger
(Удалено: спекулятивные триггеры — часть авто-генерации.)

### Requirement: Skip Silence
(Удалено: SKIP-протокол удалён; все запросы явные.)

## MODIFIED Requirements

### Requirement: Manual Trigger
Система SHALL запускать генерацию ответа только по явному запросу manual(note) из любого состояния; предыдущий незавершённый стрим SHALL отменяться (last-trigger-wins).

#### Scenario: Ручной триггер (test: manual_trigger_fires)
- GIVEN нет turn от интервьюера
- WHEN manual(Some("помоги с кодом"))
- THEN mock-сервер получил запрос с note в теле

#### Scenario: Новый запрос отменяет старый (test: manual_cancels_previous)
- GIVEN медленный mock-сервер и первый manual уже стримит
- WHEN вызван второй manual
- THEN первый стрим прерван, выполнен второй запрос

## ADDED Requirements

### Requirement: Context-Only Turns
on_turn SHALL добавлять реплику в контекст и обновлять статус, и SHALL NOT инициировать запрос к LLM.

#### Scenario: Контекст копится без генерации (test: turns_accumulate_no_fire)
- GIVEN 3 turn от Interviewer
- WHEN обработаны
- THEN mock-сервер получил 0 запросов через 1000мс

#### Scenario: Контекст в manual-запросе (test: manual_includes_context)
- GIVEN 2 turn в истории
- WHEN manual(None)
- THEN тело запроса содержит обе реплики и последнюю реплику I как фокус

## REMOVED (секция итог)
Авто-ветка кода удаляется полностью, без флага.
