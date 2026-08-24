# Proposal: Screenshots and Vision

## Why
Лайвкодинг-вопросы требуют «видеть» экран: скриншот (весь/активное окно) прикрепляется к запросу LLM; quick-action «Анализ экрана».

## What Changes
- GDI-захват экрана/активного окна (windows crate, без новых тяжёлых зависимостей)
- PNG-кодирование (crate png)
- Мультимодальные сообщения: MessageContent::Parts с image_url base64
- Orchestrator.manual(note, image_b64); команда screen_analyze
- Хоткеи screenshot_full / screenshot_region(=активное окно)

## Scope
- capture.rs в desktop (GDI BitBlt + crop)
- Изменение ChatMessage (context) на untagged-enum
- Тест: mock-сервер проверяет наличие image_url в теле запроса

## Non-Goals
- Drag-select области (MVP: «region» = активное окно)
- OCR (vision-модель вместо него)

## Affected Specs
- screenshots (ADDED), context (MODIFIED), llm (MODIFIED)
