# Report: Settings

## Что сделано
- **engine-config**: секции `[audio]` (source/mode/mic_device) и `[hotkeys]` (7 клавиш); `Config::save()` (toml::to_string_pretty); `ConfigError::Serialize`; валидация source/mode. Все struct'ы уже были Serialize (016).
- **engine-orchestrator**: `gate(mode, recording, source, is_mic)` + тесты source_gate/manual_mode_gate/vad_mode_gate.
- **hotkeys.rs**: `HotkeyRegistry`, `register_all` (unregister_all + регистрация по config), `action_for`. `dispatch(app, action)` в main.rs — manual/hide/click_through/mute/record/screenshot_full/screenshot_region. Плагин строится с `with_handler`, handler вызывает `action_for` → `dispatch`.
- **main.rs**: ConfigWatcher + перерегистрация хоткеев на `ConfigEvent::Changed`; `app.manage(watcher.config().clone())` (Arc<RwLock<Config>> для команд).
- **commands.rs**: `hotkeys_get`, `set_hotkey` (whitelist action, валидация accel через Shortcut::parse, save), `update_audio_settings` (валидация + save), `list_audio_devices` (cpal input_devices), `get_config`.
- **pipeline.rs**: gate в таске audio→segmenters (по cfg.audio.mode/source + recording flag); `start_mic_capture(device_name: Option<&str>)` с lookup по имени; `AppServices.recording: Arc<AtomicBool>`.
- **engine-audio**: `start_mic_capture(device_name)`, `mic_muted()` getter, `AudioError::Devices`.
- **app.js**: вкладка `#settings` — рендер горячих клавиш (ввод + Сохранить + отключение пустым), секция Запись (source/mode/микрофон).
- **config.toml**: добавлены секции `[audio]` и `[hotkeys]`.

## Отклонения от design.md
1. **Дефолт `manual` = "Ctrl+2"** (не "Ctrl+Shift+Space" как в design §1) — по патч-листу 017. Остальные дефолты совпадают.
2. **`dispatch` в main.rs** (а не отдельный файл) — объём маленький, связан с AppServices; design не фиксировал расположение.
3. **Добавлена команда `get_config`** — нужна settings-view для предзаполнения полей (design §6 упоминал «set_config-команду для vad», но текущая форма использует update_audio_settings + get_config).
4. **record-action шлёт `status` эмит** ("recording"/"paused") — индикация для UI; в design не описана, но без неё нет обратной связи.
5. **vad max_segment_ms через UI** — design §6 упоминал number для chunk, но смена VAD-настроек из UI требует рестарта пайплайна (VAD не hot-reload). Оставлено за кадром (только source/mode/микрофон через update_audio_settings) — см. следующий чендж при необходимости.
6. **`AudioEngine` (mic_muted)** — доступ через `mic_muted()` getter (поле приватное).

## Результаты проверок
- `cargo build --workspace` — ok; clippy `-D warnings` — 0.
- Тесты: engine-config 9 passed (включая roundtrip + validation audio); engine-orchestrator 7 passed (+3 gate); остальные зелёные. Известные флаки engine-audio (живой захват) — pre-existing.

## Осталось (manual)
- Боевая проверка: ребинд хоткея через settings (например Ctrl+K), «Отключено» пустым акселератором, смена source/mode влияет на захват, mute-toggle, record-toggle.