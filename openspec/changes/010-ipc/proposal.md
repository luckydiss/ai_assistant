# Proposal: IPC and Pipeline Wiring

## Why
Собрать все engine-крейты в работающий пайплайн внутри Tauri-приложения и пробросить события в UI.

## What Changes
- Composition root в main.rs: audio → vad(2 lane) → stt → dialogue → orchestrator → llm
- Tauri events: turn, answer_token, answer_done, status
- Tauri commands: manual_trigger
- Глобальные хоткеи (tauri-plugin-global-shortcut)

## Scope
- Wiring всех каналов
- Хоткеи: Ctrl+Shift+Space (manual), Ctrl+Shift+H (hide)
- Обновление workspace-матрицы: raw-window-handle, tauri-plugin-global-shortcut

## Non-Goals
- Tray menu, автозапуск

## Affected Specs
- ipc
