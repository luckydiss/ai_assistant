# Report: Stealth

## Что сделано
- workspace `Cargo.toml`: `windows = { version = "0.52", features = ["Win32_UI_WindowsAndMessaging", "Win32_Foundation"] }`
- `apps/desktop/src/stealth.rs`: `apply_affinity(window)` — `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)` + проверка через `GetWindowDisplayAffinity`, лог `stealth: WDA_EXCLUDEFROMCAPTURE applied`
- `apps/desktop/src/main.rs`: `mod stealth;` + `apply_affinity(...)` в setup после создания окна (main.rs:305)
- Правка применена вместе с manual-only-рейвом (код существовал, но задачи не были отмечены)

## Отклонения от design.md
1. **API raw-window-handle**: design использует `HasRawWindowHandle::raw_window_handle()`; реализация использует новый API `HasWindowHandle::window_handle()?.as_raw()` (raw-window-handle 0.6, старый метод deprecated/удалён). Поведение идентично.
2. **Обработка ошибок Windows**: design проверяет `ok.as_bool()`; реализация использует `windows::core::Result` (метод `?`) — эквивалентно.
3. Остальное совпадает с design.

## Результаты проверок
- `cargo build -p desktop` — ok
- Automated check (3.1): строка `stealth: WDA_EXCLUDEFROMCAPTURE applied` присутствовала в desktop_out.log при боевом запуске — ok
- Тесты workspace — зелёные, clippy — 0 warnings

## Осталось
- [ ] 3.2 Manual: чек-лист design.md §4 (Zoom, OBS, snipping) — нужен человек
