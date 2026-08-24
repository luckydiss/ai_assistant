# Report: Overlay UI

## Result

`apps/desktop/tauri.conf.json` — прозрачное окно-оверлей (480x640, transparent, decorations:false, alwaysOnTop, skipTaskbar, resizable:false, shadow:false, csp:null) с `withGlobalTauri: true`. Созданы `apps/desktop/ui/index.html` (карточка со статусом и ответом, скрыта по умолчанию) и `apps/desktop/ui/app.js` (vanilla JS, markdown-подмножество: code-блоки через ` ``` `, `**bold**`, `- bullets`; слушает `answer_token`/`answer_done`/`answer_skipped`/`status`/`turn`).

## Deviations from Design

1. **`withGlobalTauri` перенесён из `build` в `app`** — в Tauri v2 (schema `tauri.conf.json`) это поле живёт в `app`, а не `build`; компилятор отверг `build.withGlobalTauri` как unknown field.

2. **`frontendDist: "./ui"` вместо `"../ui"`** — дизайн предполагал `src-tauri/tauri.conf.json` (стандартный layout), но конфиг лежит в `apps/desktop/tauri.conf.json`, а UI — в `apps/desktop/ui/`. Tauri резолвит путь относительно папки конфига, поэтому `../ui` указывал бы на `apps/ui` (не существует). `./ui` → `apps/desktop/ui/`.

## Verified

- `cargo build -p desktop` — ok (proc-macro Tauri генерирует контекст, не падает)
- `cargo clippy -p desktop -- -D warnings` — ok
- `cargo fmt -p desktop --check` — ok
- `cargo build --workspace` — ok

## Not Verified (manual)

- 3.1 Окно поверх других окон
- 3.2 Клики проходят сквозь оверлей
- 3.3 Стрим-рендер + markdown

Все три — ручные проверки на живой машине. **STOP-протокол**: прозрачность на Windows WebView2 — known risk (белый фон возможен); если встретится — НЕ менять CSS наугад, сообщить человеку.