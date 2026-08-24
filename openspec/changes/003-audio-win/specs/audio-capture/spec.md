# Delta: Audio Capture

## ADDED Requirements

### Requirement: System Audio Capture
Система SHALL захватывать системный звук (все приложения) используя WASAPI loopback.

#### Scenario: Loopback capture запускается (test: captures_system_audio)
- GIVEN audio engine инициализирован
- WHEN вызван `start_system_capture()`
- THEN поток аудио данных начинается
- AND `AudioEvent::SystemData(Vec<f32>)` события отправляются

#### Scenario: Нет audio device (test: errors_on_no_device)
- GIVEN system не имеет audio output device
- WHEN вызван `start_system_capture()`
- THEN возвращается `Err(AudioError::NoDevice)`

### Requirement: Microphone Capture
Система SHALL захватывать микрофон используя стандартный input device.

#### Scenario: Mic capture запускается (test: captures_microphone)
- GIVEN audio engine инициализирован
- WHEN вызван `start_mic_capture()`
- THEN поток аудио данных начинается
- AND `AudioEvent::MicData(Vec<f32>)` события отправляются

### Requirement: Two-Lane Architecture
Система SHALL поддерживать два независимых audio lane: System (I) и Mic (C).

#### Scenario: Обе дорожки активны (test: dual_lane_capture)
- GIVEN system и mic capture запущены
- WHEN аудио приходит с обоих источников
- THEN события отправляются в правильные lane
- AND данные не смешиваются

### Requirement: Audio Format
Система SHALL конвертировать все аудио в 16kHz, mono, f32 формат.

#### Scenario: Resampling (test: resamples_to_16khz)
- GIVEN input device работает на 44.1kHz
- WHEN аудио захвачено
- THEN output data имеет sample rate 16kHz
- AND данные корректно resampled

### Requirement: Async Streaming
Система SHALL предоставлять async API для получения аудио данных.

#### Scenario: Stream subscription (test: async_stream)
- GIVEN audio capture запущен
- WHEN подписчик слушает `AudioStream`
- THEN получает `AudioEvent` события в реальном времени
- AND backpressure не блокирует capture

## MODIFIED Requirements

(none)

## REMOVED Requirements

(none)
