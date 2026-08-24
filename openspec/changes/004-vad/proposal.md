# Proposal: Voice Activity Detection

## Why
Нужно определять начало и конец речи для формирования аудио-чанков перед отправкой в STT.

## What Changes
- Silero VAD модель через ort (ONNX Runtime)
- Сегментация аудио по паузам тишины
- Максимальная длина сегмента 7 секунд
- Async API для обработки аудио потока

## Scope
- Загрузка Silero VAD ONNX модели
- Обработка аудио чанков 16kHz mono f32
- Сегментация по тишине (настраиваемая пауза)
- Принудительное разбиение длинных сегментов
- Streaming API для real-time обработки

## Non-Goals
- Speaker diarization (разделение спикеров)
- Audio processing (noise reduction, gain)
- Multi-language VAD (используем универсальный Silero)

## Affected Specs
- vad
