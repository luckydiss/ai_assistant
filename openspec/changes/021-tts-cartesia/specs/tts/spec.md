# TTS Specification (полная замена домена)

## Purpose
Real-time озвучка ответов ассистента через Cartesia WebSocket: звук начинается с первого предложения до завершения генерации; управление режимами off/auto/hotkey.

## Requirements

### Requirement: Streaming Synthesis
Система SHALL открывать WS-сессию Cartesia на один ответ, отправлять предложения по мере их готовности и принимать аудио-чанки pcm_f32le 22050 до finalize.

#### Scenario: Звук раньше конца генерации (test: first_audio_before_done)
- GIVEN mock WS-сервер отдаёт chunk на первое предложение
- WHEN отправлено первое предложение, LLM ещё стримит
- THEN TtsOut::Pcm получен до завершения ответа

#### Scenario: Финализация (test: finalize_sent)
- WHEN отправлен Flush
- THEN mock получил финальное сообщение с "continue": false и пустым transcript

#### Scenario: Контекст на ответ (test: context_per_answer)
- GIVEN две последовательные сессии
- THEN context_id различаются

### Requirement: Sentence Feeder
Система SHALL накапливать токены, вырезать ```-блоки и отдавать готовые предложения; остаток отдаётся по finish.

#### Scenario: Резка по предложениям (test: feeder_splits_sentences)
- GIVEN токены "Привет! Как де" + "ла? Код."
- WHEN push_token
- THEN после первого: ["Привет!"], после второго: ["Как дела?", "Код."]

#### Scenario: Код не озвучивается (test: feeder_skips_code)
- GIVEN токены содержат "```python\nprint(1)\n```"
- WHEN обработаны
- THEN содержимое блока не попадает ни в одно предложение

#### Scenario: Хвост по finish (test: feeder_flush_tail)
- GIVEN буфер "без точки"
- WHEN finish()
- THEN ["без точки"]

### Requirement: Playback
Система SHALL проигрывать f32-чанки через cpal в порядке поступления, с линейным ресемплом 22050 → rate устройства.

#### Scenario: Ресемпл (test: resample_length_ratio)
- GIVEN 22050→44100
- WHEN resample_linear_f32(1000 сэмплов)
- THEN длина ≈ 2000 (±2)

#### Scenario: Порядок чанков (test: playback_order)
- GIVEN push(A) затем push(B)
- THEN в очереди A перед B

#### Scenario: Очистка (test: player_clear)
- WHEN clear()
- THEN очередь пуста

### Requirement: Modes
[tts] mode: "off" — тишина; "auto" — стриминг на каждый ответ; "hotkey" — озвучка последнего ответа по Ctrl+T.

#### Scenario: Auto (test: manual_auto_mode)
- WHEN mode=auto и идёт ответ
- THEN звук начинается до конца генерации (manual)

#### Scenario: Hotkey (test: manual_hotkey_mode)
- WHEN mode=hotkey, ответ готов, нажат Ctrl+T
- THEN озвучивается последний ответ; повторный Ctrl+T глушит (manual)

#### Scenario: Off (test: manual_off_mode)
- WHEN mode=off
- THEN запросов к Cartesia нет (manual/логи)

### Requirement: Cancellation
Новый ответ, повторный хоткей или Stop SHALL прерывать WS-сессию и чистить очередь плеера.

#### Scenario: Отмена предыдущей (test: cancel_previous_session)
- GIVEN активная сессия и проигрывание
- WHEN стартует новый ответ
- THEN старая сессия прервана, очередь пуста, начата новая

### Requirement: TTS Config
Система SHALL читать [tts]: mode, provider, api_key, model_id, voice_id, sample_rate; дефолты: mode=off, provider=cartesia, model_id=sonic-3.5, voice_id=1e4176b1-3db9-44d6-a601-4fe68b041942, sample_rate=22050. api_key пустой при mode!=off → ConfigError::Validation.

#### Scenario: Дефолты (test: tts_defaults)
#### Scenario: Ключ обязателен (test: tts_requires_key)
- GIVEN mode=auto, api_key=""
- WHEN load
- THEN Err(Validation)

#### Scenario: Невалидный mode (test: tts_validates_mode)

### Requirement: UI Speaker Button
Оверлей SHALL иметь кнопку-динамик, эквивалентную хоткею tts (озвучить последний ответ / стоп).

#### Scenario: Кнопка (test: manual_speaker_button)

## REMOVED Requirements

### Requirement: Synthesis (019, wav через /audio/speech)
(Заменено стримингом Cartesia.)

### Requirement: Playback (019, wav/hound)
(Заменено f32-плеером.)