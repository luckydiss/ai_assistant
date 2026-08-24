# Proposal: Windows Audio Capture

## Why
Нужен захват системного звука (интервьюер) и микрофона (кандидат) как две отдельные дорожки.

## What Changes
- cpal-based audio capture с WASAPI loopback
- Две отдельные audio lanes
- Асинхронный стриминг аудио чанков

## Scope
- WASAPI loopback для system audio
- Стандартный input device для микрофона
- 2-lane architecture (I = interviewer, C = candidate)
- Audio format: 16kHz, mono, f32
- Async stream API

## Non-Goals
- Audio processing (это в engine-vad)
- Audio playback
- Multi-channel audio

## Affected Specs
- audio-capture
