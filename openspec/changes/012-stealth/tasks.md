# Tasks: Stealth

- [x] 1.1 Обновить features `windows` в workspace Cargo.toml по design.md §1
  verify: `cargo build -p desktop` — ok
  features: `Win32_UI_WindowsAndMessaging`, `Win32_Foundation`

- [x] 1.2 Добавить raw-window-handle в apps/desktop/Cargo.toml (уже в workspace)
  verify: `cargo build -p desktop` — ok

- [x] 2.1 Создать `apps/desktop/src/stealth.rs` по design.md §2
  verify: `cargo build -p desktop` — ok
  `apply_affinity` ставит WDA_EXCLUDEFROMCAPTURE и проверяет через GetWindowDisplayAffinity

- [x] 2.2 Подключить в main.rs по design.md §3
  verify: `cargo build -p desktop` — ok
  `apply_affinity` вызывается в конце setup (main.rs:305)

- [x] 3.1 Automated check: запустить приложение, в логах "stealth: WDA_EXCLUDEFROMCAPTURE applied"
  verify: строка присутствовала в desktop_out.log при запуске — ok

- [ ] 3.2 Manual: чек-лист design.md §4 (Zoom, OBS, snipping)
  verify: оверлей не виден в захвате во всех трёх
  Нужен пользователь: проверить в реальном Zoom/OBS/ncnipping, что оверлей не попадает в захват.

## STOP Protocol
Если SetWindowDisplayAffinity возвращает false — проверить Windows 10 20H1+; НЕ пытаться fallback на WDA_MONITOR. Остановиться и спросить.