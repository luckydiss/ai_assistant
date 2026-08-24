# Change TTS Cartesia Streaming

## Why
Текущая озвучка (019) ждёт полной генерации ответа и только потом синтезирует — слишком долго. Нужен real-time: звук начинается с первого предложения, пока ответ ещё генерируется. Провайдер — Cartesia (sonic-3.5, русский голос), WebSocket-стриминг.

## What Changes
- Новый crate engine-tts: sentence-фидер, Cartesia WS-клиент, f32-плеер (cpal)
- Сессия TTS на один ответ: push предложений → finalize → приём аудио-чанков
- Режимы [tts] mode: off | auto | hotkey; хоткей Ctrl+T; кнопка-динамик в оверлее
- Отмены: новый ответ / повторный Ctrl+T / Stop глушат сессию и очередь
- Config: [tts] (api_key только из конфига), хоткей tts в [hotkeys]
- Удалить старую озвучку 019 (TtsClient gemini/wav/hound)

## Scope
- sample_rate 22050, pcm_f32le, container raw
- Код не озвучивается (```-блоки вырезаются)
- Линейный ресемпл 22050 → rate устройства

## Non-Goals
- 44100/48000 на входе TTS (фиксировано 22050)
- HTTP non-stream путь, клонирование голоса, перевод озвучки
- Стриминг входа (микрофон→TTS)

## Security
API-ключ ТОЛЬКО из config.toml (файл в .gitignore). Хардкод ключа в коде = нарушение STOP-протокола.

## Affected Specs
- tts (полная замена домена), config (ADDED [tts] и хоткей), ui (ADDED кнопка)