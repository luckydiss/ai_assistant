# Delta: Settings

## ADDED Requirements

### Requirement: Configurable Hotkeys
Система SHALL читать биндинги из [hotkeys] и позволять менять их командой set_hotkey(action, accel) с валидацией.

#### Scenario: Валидный ребинд (test: set_hotkey_valid)
- GIVEN set_hotkey("hide", "Ctrl+B")
- WHEN hotkeys_get()
- THEN hide = "Ctrl+B" И файл config.toml содержит новое значение

#### Scenario: Невалидный акселератор (test: set_hotkey_invalid)
- GIVEN set_hotkey("hide", "NotAKey")
- WHEN вызов
- THEN Err, старое значение сохранено

#### Scenario: Отключение (test: set_hotkey_disabled)
- GIVEN set_hotkey("mute", "")
- WHEN hotkeys_get
- THEN mute = "" и хоткей не зарегистрирован

### Requirement: Audio Source Mode
Система SHALL захватывать только lane'ы, разрешённые [audio] source: "system+mic" | "system" | "mic".

#### Scenario: Gate-функция (test: source_gate)
- GIVEN source="system"
- WHEN gate(lane=Mic)
- THEN false; gate(lane=System) = true

### Requirement: Manual Recording Mode
В режиме mode="manual" аудио обрабатывается только пока recording=true (тумблер/хоткей record).

#### Scenario: Gate manual (test: manual_mode_gate)
- GIVEN mode="manual", recording=false
- WHEN gate(любой lane)
- THEN false; при recording=true → true

#### Scenario: VAD-режим без гейта (test: vad_mode_gate)
- GIVEN mode="vad"
- WHEN gate
- THEN всегда true (mute микрофона учитывается отдельно)

### Requirement: Mic Device Selection
Система SHALL перечислять микрофоны и использовать выбранный в [audio] mic_device.

#### Scenario: Список устройств (test: lists_mic_devices)
- WHEN list_audio_devices()
- THEN возвращает непустой список inputs (на машине с микрофоном)

#### Scenario: Старт с выбранным микрофоном (test: manual_mic_selected)
- GIVEN mic_device = имя из списка
- WHEN start_pipeline
- THEN логи содержат это имя устройства (manual)

### Requirement: Settings View
Main-окно SHALL показывать view «Настройки»: секции Запись (source, mode, mic, длина чанка) и Управление (список хоткеев с полем ребинда и состоянием «Отключено»).

#### Scenario: UI ребинд (test: manual_settings_ui)
- WHEN в поле введено "Ctrl+K" и сохранено
- THEN хоткей работает, после перезапуска сохраняется (manual)

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
