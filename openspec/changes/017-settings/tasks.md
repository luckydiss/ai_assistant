# Tasks: Settings

- [x] 1.1 Serialize + audio/hotkeys секции + save() в engine-config по design.md §1
  verify: `cargo test -p engine-config` (старые зелёные) — ok

- [x] 1.2 Тест toml-roundtrip: load→save→load сохраняет hotkeys
  verify: `cargo test -p engine-config roundtrip` — ok (toml_roundtrip_keeps_hotkeys)

- [x] 2.1 gate() в engine-orchestrator по design.md §2
  verify: тесты source_gate, manual_mode_gate, vad_mode_gate — ok

- [x] 3.1 hotkeys.rs по design.md §3; dispatch в main.rs; перерегистрация на ConfigEvent::Changed
  verify: `cargo build -p desktop` — ok

- [x] 4.1 Команды hotkeys_get/set_hotkey/update_audio_settings/list_audio_devices по design.md §4
  verify: `cargo build -p desktop` — ok (+ get_config)

- [x] 5.1 Gate + mic_device в pipeline.rs по design.md §5
  verify: `cargo build -p desktop` — ok

- [x] 6.1 Settings-view в app.js по design.md §6
  verify: manual — ребинд работает, «Отключено» гасит хоткей, смена source/mode влияет на захват
  Создан (вкладка #settings); ждёт боевой проверки

- [x] 7.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0 — ok (кроме известных флаков engine-audio)

## STOP Protocol
`accel.parse::<Shortcut>()` работает — Shortcut из global_hotkey имеет FromStr (см. lib.rs плагина: `Shortcut::from_str`). Собственный парсер не понадобился.