# Proposal: Meetings and Contexts

## Why
Продукту нужны первоклассные сущности «встреча» (сессия с метриками и resume) и «контекст» (роль/резюме/доп. промпт), управляемые из UI и используемые ContextBuilder.

## What Changes
- sqlite: таблицы meetings и contexts; start_session становится upsert (resume)
- CRUD-команды Tauri для meetings/contexts
- Импорт резюме TXT/MD (файл читает фронтенд, передаёт текст)
- ContextBuilder строит промпт из активного контекста: role→persona, resume+extra+vacancy→system

## Scope
- store-методы + IPC-команды (headless, тестируемо)
- Интеграция с ContextBuilder

## Non-Goals
- PDF-парсинг (позже), переводы на 2–3 языка (020), UI-рендер (016)

## Affected Specs
- workspace (ADDED), store (MODIFIED), context (MODIFIED)
