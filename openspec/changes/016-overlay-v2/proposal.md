# Proposal: Two-Window UI (Overlay v2)

## Why
Заменить карточку 011 полноценным оверлеем-чатом и панелью управления (встречи, контексты) по образцу sobes.tech.

## What Changes
- Два окна: main (панель) + overlay (невидимый чат)
- Пайплайн стартует/останавливается на встречу (команды start/stop)
- Оверлей: чат транскрипта+ответов, quick-actions «Что сказать»/«Резюме», ручной ввод, mute, статус-схема VAD, бейджи модели и защиты, click-through
- Main: view «Встречи» (список/создание/continue/удаление/поиск) и «Контексты» (редактор)
- engine-vad: события состояний (Ожидание/Запись/Пауза/Отправка)
- engine-audio: mute микрофона

## Scope
- pipeline.rs (вынос wiring из main.rs в start/stop)
- overlay.html/overlay.js, index.html/app.js (views)
- Команды: start_pipeline, stop_pipeline, mic_mute, protection_status, click_through

## Non-Goals
- Конфигурируемые хоткеи (017), скриншоты/vision (018), TTS (019), переключатель моделей (показываем модель из конфига)

## Affected Specs
- ui (MODIFIED), ipc (MODIFIED), vad (MODIFIED), audio-capture (MODIFIED)
