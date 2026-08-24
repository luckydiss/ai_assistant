# Proposal: Dialogue Assembler

## Why
Нужно собирать транскрипты из двух audio lane (интервьюер и кандидат) в единый диалог с правильной последовательностью.

## What Changes
- Reorder буфер для компенсации задержек STT
- Склейка обрывков одной фразы
- Дедупликация повторяющихся транскриптов
- Фильтрация галлюцинаций Whisper
- Rolling summary для длинных диалогов

## Scope
- Timeline-based assembly с timestamp ordering
- Merge logic для коротких пауз (< 500мс)
- Dedup для одинаковых текстов
- Garbage filter для коротких/бессмысленных реплик
- Background summarization каждые 16 turns

## Non-Goals
- Speaker diarization (уже разделено в audio lane)
- Translation (работаем с оригинальным языком)
- Sentiment analysis

## Affected Specs
- dialogue
