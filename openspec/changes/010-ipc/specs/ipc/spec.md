# Delta: IPC

## ADDED Requirements

### Requirement: Pipeline Wiring
Система SHALL соединять модули: audio events → segmenters → stt → assembler → orchestrator.

#### Scenario: Полный пайплайн (test: manual_e2e_smoke)
- GIVEN запущенное приложение и играющий системный звук с речью
- WHEN речь распознана
- THEN в UI приходит turn-событие и answer-события (manual check)

### Requirement: UI Events
Система SHALL эмитировать Tauri-события: "turn", "answer_token", "answer_done", "answer_skipped", "status".

#### Scenario: События доходят до фронтенда (test: events_reach_frontend)
- GIVEN открыт devtools фронтенда
- WHEN пайплайн активен
- THEN window.__TAURI__.event.listen получает события (manual check)

### Requirement: Hotkeys
Система SHALL реагировать на глобальные хоткеи даже когда окно не в фокусе.

#### Scenario: Ручной триггер (test: hotkey_manual)
- GIVEN фокус в Zoom
- WHEN нажат Ctrl+Shift+Space
- THEN orchestrator.manual вызван (manual check: статус "generating")

#### Scenario: Скрыть оверлей (test: hotkey_hide)
- WHEN нажат Ctrl+Shift+H
- THEN оверлей скрыт; повторное нажатие показывает (manual check)

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
