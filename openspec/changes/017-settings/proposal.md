# Proposal: Configurable Settings

## Why
Пользователь должен управлять хоткеями (включая отключение), источником звука, режимом записи и микрофоном — как в sobes.tech.

## What Changes
- config: секции [audio] (source, mode, mic_device) и [hotkeys] (7 биндингов, "" = отключено)
- Hotkey-менеджер: регистрация из конфига, перерегистрация при hot-reload
- Команды: set_hotkey, hotkeys_get, update_audio_settings, list_audio_devices
- Gate-логика: manual-режим записи (Ctrl+R старт/стоп), source-режимы (system/mic/both)
- Settings-view в main-окне

## Scope
- Конфиг + serde-roundtrip (Serialize для записи файла)
- Перерегистрация хоткеев на ConfigEvent::Changed
- Выбор микрофона (cpal input_devices); выбор output-устройства — Non-Goal

## Non-Goals
- Выбор output-устройства для loopback (позже)
- UI-редактор с «нажмите клавишу» (ввод строкой акселератора)

## Affected Specs
- settings (ADDED), config (MODIFIED), ipc (ADDED)
