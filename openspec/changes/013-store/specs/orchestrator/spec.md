# Delta: Orchestrator (pure trigger decision)

## ADDED Requirements

### Requirement: Pure Trigger Decision
Система SHALL выделять чистую функцию trigger_decision(turn, min_words, debounce_ms) -> Option<TriggerKind>, используемую и в live-цикле, и в replay.

#### Scenario: Интервьюер, достаточно слов (test: decision_auto)
- GIVEN turn от Interviewer, 10 слов, без "?"
- WHEN trigger_decision(min_words=4)
- THEN Some(Auto)

#### Scenario: Спекулятивный (test: decision_speculative)
- GIVEN turn от Interviewer, 20 слов, с "?"
- WHEN trigger_decision
- THEN Some(Speculative)

#### Scenario: Кандидат или короткие реплики (test: decision_none)
- GIVEN turn от Candidate ИЛИ 3 слова
- WHEN trigger_decision
- THEN None

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
