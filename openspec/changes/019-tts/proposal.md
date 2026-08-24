# Proposal: Answer Voice-Over (TTS)

## Why
Озвучка ответов позволяет «проговорить» подсказку, не читая с экрана (сценарий созвонов).

## What Changes
- TtsClient: POST {base_url}/audio/speech (та же OpenAI-compatible точка), response_format=wav
- Проигрывание через cpal output; отмена предыдущего проигрывания
- config [tts] enabled/voice/model; тумблер в настройках; авто-озвучка на AnswerDone

## Scope
- engine-llm: TtsClient (synth_wav)
- desktop: player (cpal) + wiring на OrchEvent::Done
- Команда tts_toggle

## Non-Goals
- Стриминговый TTS (только целый ответ)
- Кастомные голоса вне списка endpoint

## Affected Specs
- tts (ADDED)
