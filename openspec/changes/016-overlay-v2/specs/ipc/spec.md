# Delta: IPC (pipeline lifecycle)

## MODIFIED Requirements

### Requirement: Pipeline Wiring
Пайплайн SHALL запускаться и останавливаться на встречу командами start_pipeline(meeting_id) / stop_pipeline(); wiring вынесен из setup в модуль pipeline.rs. При старте: start_session(meeting_id), активный контекст встречи → ContextBuilder::with_workspace.

#### Scenario: Старт на встречу (test: manual_start_per_meeting)
- WHEN start_pipeline("m1")
- THEN аудио захватывается, turns пишутся в встречу m1 (manual)

#### Scenario: Стоп (test: manual_stop)
- WHEN stop_pipeline()
- THEN стримы остановлены, end_session вызван (manual)

## ADDED Requirements

### Requirement: UI Commands
Система SHALL предоставлять команды: mic_mute(bool), protection_status() -> bool, click_through(bool), vad_state-события "vad", overlay-события "turn"/"answer_*" (существуют).

#### Scenario: protection_status (test: protection_status_true)
- GIVEN affinity применён
- WHEN вызвана команда
- THEN true (automated, windows-only)

## REMOVED Requirements
(none)
