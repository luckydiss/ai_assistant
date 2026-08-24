# Delta: TTS

## ADDED Requirements

### Requirement: Synthesis
Система SHALL синтезировать речь через POST /audio/speech с response_format="wav" и возвращать валидный WAV.

#### Scenario: WAV из mock (test: tts_returns_wav)
- GIVEN mock-сервер отдаёт байты валидного wav
- WHEN synth_wav("привет")
- THEN возвращены байты с RIFF-заголовком

#### Scenario: Ошибка endpoint (test: tts_error_surfaced)
- GIVEN mock отдаёт 500
- WHEN synth_wav
- THEN Err, без паники

### Requirement: Playback
Система SHALL проигрывать WAV через дефолтное output-устройство; новый ответ отменяет предыдущее проигрывание.

#### Scenario: Отмена предыдущего (test: playback_cancels_previous)
- GIVEN играет длинный wav
- WHEN speak() второй раз
- THEN первый поток остановлен (один активный stream)

### Requirement: Toggle and Auto-Read
[tts] enabled=true SHALL озвучивать каждый AnswerDone; команда tts_toggle меняет флаг в конфиге.

#### Scenario: Тумблер (test: manual_tts_toggle)
- WHEN tts выключен тумблером
- THEN ответы не озвучиваются; включён — озвучиваются (manual)

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
