# Proposal: Invisible Overlay UI

## Why
Нужно окно-подсказка: прозрачное, always-on-top, не мешает кликам вне карточки, стримит markdown.

## What Changes
- Tauri window config: transparent, decorations false, alwaysOnTop, skipTaskbar
- Статический фронтенд без npm-сборки (withGlobalTauri)
- Минимальный markdown-рендерер (code fences, буллеты, bold)
- Стрим-рендер токенов

## Scope
- ui/index.html + ui/app.js (без node-сборки)
- Window config
- Статус-индикатор

## Non-Goals
- React/Vite-сборка (позже, когда пайплайн стабилен)
- Drag&drop, настройки в UI

## Scope Notes
Без npm: фронтенд — статика, использует window.__TAURI__.

## Affected Specs
- ui
