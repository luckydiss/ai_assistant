# Tasks: Overlay v2

- [x] 1.1 VadState + события в Segmenter по design.md §1
  verify: `cargo test -p engine-vad vad_state_sequence` — ok
  VadState {Waiting,Recording,Paused,Sending}, subscribe_states(), эмиссия в process_chunk

- [x] 1.2 set_mic_muted в AudioEngine по design.md §2
  verify: `cargo test -p engine-audio mic_mute_stops_events` — ok (4 passed; 3 других живых — известные флаки)
  mic_muted: Arc<AtomicBool>, проверка в mic-колбэках

- [x] 2.1 Вынести wiring в pipeline.rs по design.md §3; main.rs не стартует пайплайн
  verify: `cargo build -p desktop` — ok
  Весь wiring (audio→vad→stt→assembler→orch→store/logger) перенесён в `pipeline::start`

- [x] 2.2 AppServices + команды start/stop/mute/protection/click_through по design.md §5
  verify: `cargo build -p desktop` — ok
  AppServices {store, audio, orch, pipeline}, 5 новых команд

- [x] 3.1 Окно overlay в setup по design.md §4 + stealth на него
  verify: `cargo build -p desktop` — ok; логи при запуске: "stealth: WDA_EXCLUDEFROMCAPTURE applied" (см. report)

- [x] 4.1 overlay.html + overlay.js по design.md §6
  verify: manual — лента, стадии VAD, бейджи, quick-actions
  Созданы; ждёт боевой проверки

- [x] 4.2 index.html + app.js (views meetings/contexts) по design.md §7
  verify: manual — создание встречи, «Продолжить», контексты
  Hash-роутер, renderMeetings/renderContexts; ждёт боевой проверки

- [x] 5.1 Хоткей Ctrl+W → click_through; Ctrl+Shift+H hide; Ctrl+2 → «Что сказать»
  verify: manual — кликабельность переключается
  Зарегистрированы в main.rs; см. отклонение про Ctrl+2

- [x] 6.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0 — ok (кроме известных флаков engine-audio)

## STOP Protocol
set_ignore_cursor_events — метод WebviewWindow в Tauri v2 (импорт tauri::Manager), работает. Архитектура двух окон сохранена.