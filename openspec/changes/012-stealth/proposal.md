# Proposal: Stealth (window exclusion from capture)

## Why
Оверлей не должен попадать в захват экрана Zoom/Meet/OBS.

## What Changes
- SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE) для главного окна при старте
- Автоматическая проверка через GetWindowDisplayAffinity
- Ручной чек-лист самопроверки захвата

## Scope
- stealth-модуль в desktop: apply_affinity(hwnd)
- raw_window_handle для получения HWND
- Manual verification checklist

## Non-Goals
- Автоматический DXGI pixel-probe (сложно, отложено)
- Скрытие процесса из Task Manager

## Affected Specs
- stealth
