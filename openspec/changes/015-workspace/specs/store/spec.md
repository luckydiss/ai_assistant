# Delta: Store (resume support)

## MODIFIED Requirements

### Requirement: Session Persistence
Система SHALL сохранять сессию, turns и ответы в локальный sqlite. start_session SHALL быть upsert: повторный вызов с тем же id не создаёт дубликат и сбрасывает ended_at.

#### Scenario: Upsert сессии (test: start_session_upsert)
- GIVEN start_session("m1") затем end_session("m1")
- WHEN start_session("m1") снова
- THEN в sessions одна строка с ended_at = NULL

(Прежние сценарии creates_session_and_turns / records_answer_metrics сохраняются.)

## ADDED Requirements
(none)

## REMOVED Requirements
(none)
