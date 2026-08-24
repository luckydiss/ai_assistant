# Delta: E2E Acceptance (v2, полная замена)

## ADDED Requirements

### Requirement: Core Acceptance
#### Scenario: Мок-собеседование 40 минут (test: manual_mock_interview)
- GIVEN Zoom + интервьюер: 15 вопросов теории, 2 лайвкодинга, 3 smalltalk-ловушки
- WHEN прогон с ассистентом
- THEN ≥ 80% вопросов получили релевантный ответ; ≥ 90% ловушек ушли в SKIP

#### Scenario: Латентности (test: manual_latency_budget)
- GIVEN sqlite stats
- THEN p50(ttft) ≤ 1800мс, p95(ttft) ≤ 3000мс

### Requirement: Sessions Acceptance
#### Scenario: Встречи и continue (test: manual_meetings)
- WHEN создана встреча, пройден прогон, приложение перезапущено и нажато «Продолжить»
- THEN история на месте, счётчик сообщений дописывается, новая сессия легла в ту же встречу

#### Scenario: Контексты влияют (test: manual_contexts)
- GIVEN контекст A (role: backend) и B (role: frontend, extra-промпт)
- WHEN переключены между двумя короткими прогонами
- THEN стиль/содержание ответов различимо меняется

### Requirement: Settings Acceptance
#### Scenario: Ребинд и отключение (test: manual_hotkeys)
- WHEN hide перебинден на Ctrl+K и mute отключён ("")
- THEN Ctrl+K скрывает окно, Ctrl+M не срабатывает; после перезапуска сохраняется

#### Scenario: Режимы записи (test: manual_modes)
- GIVEN source="mic" — системная речь игнорируется; mode="manual" — запись только по Ctrl+R
- WHEN проверено
- THEN поведение совпадает

#### Scenario: Устройство микрофона (test: manual_mic_device)
- WHEN выбран не-дефолтный микрофон
- THEN в логах старта его имя

### Requirement: Vision/TTS/Translations Acceptance
#### Scenario: Лайвкодинг со скриншотом (test: manual_vision)
- WHEN во время лайвкодинга нажат Ctrl+H
- THEN ответ соответствует коду на экране

#### Scenario: TTS (test: manual_tts)
- WHEN tts включён
- THEN ответ озвучен; при выключенном — тишина; новый ответ прерывает предыдущую озвучку

#### Scenario: Переводы (test: manual_translations)
- GIVEN контекст ["ru","en"]
- THEN под ответом появляется английский перевод с кодом без перевода идентификаторов

### Requirement: Overlay UX Acceptance
#### Scenario: Управление окном (test: manual_overlay_controls)
- WHEN Ctrl+W / Ctrl+B / mute / «Что сказать» / «Резюме»
- THEN кликабельность переключается, окно скрывается/показывается, C-реплики глушатся, действия срабатывают

#### Scenario: Стадии VAD и бейджи (test: manual_indicators)
- WHEN идёт речь
- THEN индикатор проходит ожидание→запись→пауза→отправка; бейджи «Защита вкл.» и модель видны

### Requirement: Stealth Acceptance
#### Scenario: Тройная проверка (test: manual_stealth_triple)
- WHEN Zoom share + OBS display capture + Win+Shift+S одновременно
- THEN оверлей не виден ни в одном захвате

### Requirement: Resource Acceptance
#### Scenario: Ресурсы (test: manual_resources)
- THEN CPU ≤ 10% среднее, RAM ≤ 300МБ, вентиляторы не стартуют за 40 минут

## MODIFIED / REMOVED Requirements
(прежние требования e2e v1 заменяются этим набором)
