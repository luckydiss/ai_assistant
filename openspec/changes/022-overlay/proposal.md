# Proposal: Overlay GUI 1:1 (sobes layout)

## Why
Привести оверлей к layout sobes 1:1: rail чатов, топбар с дропдаунами, группы-чипы, нижняя тулбар с тумблерами, окно-индикатор статусов, прозрачность/акцент из конфига.

## What Changes
- Overlay: левый rail чатов (номера, «+»), топбар (mute, STT-дропдаун, модель, заметки-view, домой), лента с группами «Расшифровка аудио (N)» и «Инструменты (N)», quick-actions, инпут, тулбар (чаты, контекст, скриншоты, TTS, автоответы, функции ИИ, сброс контекста)
- Чаты: отдельные истории на встречу; у чата свой контекст
- Окно-индикатор статусов (отдельное stealth-окно, угол по умолчанию сверху справа)
- Engine: multi-chat в orchestrator/assembler; [stt] language; [llm] search_enabled + search_tool_json (tool у endpoint); store: таблицы chats, notes
- Config: секции [ui], [window], [chat] с дефолтами (UI-редакция этих настроек — в 023/024)

## Scope
- Заметки: dropdown = только просмотр (CRUD и RAG — 024)
- tglNotesRag персистит флаг; RAG-пайплайн — 024
- Модель-дропдаун: read-only текущая модель (профили — позже)
- Хоткеи перемещения/ресайза окна — 024

## Non-Goals
- System design режим, drag-select области, профили моделей

## Affected Specs
- ui (MODIFIED), orchestrator (MODIFIED), stt (MODIFIED), llm (MODIFIED), store (ADDED), config (ADDED)
