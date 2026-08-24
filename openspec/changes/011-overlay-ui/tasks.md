# Tasks: Overlay UI

- [x] 1.1 Заменить `apps/desktop/tauri.conf.json` по design.md §1
  verify: `cargo build -p desktop`

- [x] 2.1 Создать `apps/desktop/ui/index.html` по design.md §2
  verify: файл существует

- [x] 2.2 Создать `apps/desktop/ui/app.js` по design.md §3
  verify: файл существует

- [ ] 3.1 Manual: запустить приложение, карточка поверх других окон
  verify: карточка видна поверх Zoom/браузера

- [ ] 3.2 Manual: клик мимо карточки проходит в другое приложение
  verify: клик срабатывает в приложении под оверлеем

- [ ] 3.3 Manual: стрим-рендер и markdown (code/bullets/bold)
  verify: ответ появляется постепенно, код в pre-блоке

## STOP Protocol
Если прозрачность не работает на Windows (белый фон) — НЕ менять CSS наугад; сообщить человеку (_known WebView2 risk_).
Не добавлять npm-зависимости. Остановиться и спросить.