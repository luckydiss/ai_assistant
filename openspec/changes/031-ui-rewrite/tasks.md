# Tasks: UI Rewrite (change 031)

## Phase 0: Preparation & Audit (1 week)

- [ ] 0.1 Аудит текущего UI: каталог компонентов, зависимостей, состояния
  verify: `docs/ui/audit/components.md` содержит таблицу с 10+ компонентами

- [ ] 0.2 Baseline метрики: bundle size, paint time, lines of code, a11y score
  verify: `docs/ui/audit/metrics-baseline.md` заполнен реальными числами
  (Chrome DevTools Performance, `ls -lh ui/*.{js,css}`, axe-core CLI)

- [ ] 0.3 Приоритизация болей (P0/P1/P2) с метриками
  verify: `docs/ui/audit/pain-points.md` содержит чеклист с severity labels

- [ ] 0.4 Выбор стека: TS + Svelte 5 + Vite 6 + TailwindCSS 4 (задокументировано в proposal.md)
  verify: `proposal.md` §"Стек" заполнен с обоснованием vs альтернатив

- [ ] 0.5 Setup `ui-next/`: Vite + Svelte + TypeScript + TailwindCSS
  verify: `npm create vite@latest ui-next -- --template svelte-ts` + `npm run dev` открывает пустую страницу

- [ ] 0.6 POC: TtsControls.svelte — простой изолированный компонент
  verify: toggle работает идентично старому, `npm run build` → bundle < 3kb gzip

- [ ] 0.7 POC: интеграция Tauri invoke() из Svelte компонента
  verify: `commands.ttsGet()` возвращает данные от Rust backend в dev mode

## Phase 1: Foundation (2-3 weeks)

### 1.1 TypeScript + Build System

- [ ] 1.1.1 `tsconfig.json` strict mode, ES2020 target, path aliases (`$lib/*`)
  verify: `npx tsc --noEmit` проходит без ошибок на пустом проекте

- [ ] 1.1.2 ESLint + Prettier конфигурация
  verify: `npm run lint` выход 0 на стартовом коде

- [ ] 1.1.3 Pre-commit hook (lint-staged) блокирует commit с ошибками
  verify: `git commit` с unused variable → rejected

- [ ] 1.1.4 Bundle size budget check в CI (≤80kb gzip)
  verify: скрипт `scripts/check-bundle-size.js` фейлится при превышении

### 1.2 Tauri Specta Integration

- [ ] 1.2.1 Добавить `tauri-specta`, `specta`, `specta-typescript` в `Cargo.toml`
  verify: `cargo build -p desktop` компилируется с новыми зависимостями

- [ ] 1.2.2 Аннотировать `#[specta::specta]` на всех Tauri командах (~40 команд)
  verify: `cargo build -p desktop` без ошибок, все команды в списке `collect_commands!`

- [ ] 1.2.3 `#[derive(Type)]` на всех структурах, пересекающих Rust↔TS границу
  (ModelMetadata, Config, Turn, ChatSection, UiSection, ...)
  verify: `cargo build -p desktop` генерирует `ui-next/src/lib/bindings.ts` без ошибок

- [ ] 1.2.4 build.rs генерирует `bindings.ts` при каждой компиляции (debug mode)
  verify: изменение сигнатуры команды → пересборка → `bindings.ts` обновлён

- [ ] 1.2.5 Fallback build.rs: `ui-next/dist` существует → используется, иначе `ui/`
  verify: удаление `ui-next/dist` → `cargo build` использует старый `ui/`

### 1.3 Design System

- [ ] 1.3.1 `tokens.css`: colors, spacing, typography, radius, shadows, z-index
  verify: файл содержит все переменные из design.md §"Tokens"

- [ ] 1.3.2 `tailwind.config.ts` extends tokens через CSS variables
  verify: `class="bg-bg-primary"` компилируется в `background-color: var(--color-bg-primary)`

- [ ] 1.3.3 Button.svelte: variants (primary/secondary/ghost/danger), sizes, loading, disabled
  verify: unit test `Button.test.ts` — 4 варианта, 3 размера, disabled/loading states

- [ ] 1.3.4 Modal.svelte: focus trap, Escape close, backdrop click, ARIA (role=dialog)
  verify: unit test — focus cycles Tab, Escape закрывает, axe-core 0 violations

- [ ] 1.3.5 Input.svelte: controlled value, placeholder, label animation, error state
  verify: unit test — value обновляется на input event, error class применяется

- [ ] 1.3.6 Slider.svelte: min/max/step, keyboard arrows, visual track fill
  verify: unit test — ArrowUp/Down изменяют value кратно step

- [ ] 1.3.7 Select.svelte, Checkbox.svelte, Badge.svelte, Toast.svelte
  verify: unit tests для каждого, axe-core 0 violations

- [ ] 1.3.8 Dark mode tokens (`:root:not([data-theme="dark"])` light overrides)
  verify: `data-theme="dark"` toggle меняет computed styles (manual visual test)

- [ ] 1.3.9 Lucide icons: только используемые импортированы (tree-shaking проверка)
  verify: `npx vite-bundle-visualizer` показывает lucide chunk ≤5kb gzip

### 1.4 State Management

- [ ] 1.4.1 `config.svelte.ts`: initial load, `config_changed` event listener, optimistic updates
  verify: unit test — store обновляется на event, rollback на ошибку

- [ ] 1.4.2 Rust: `ConfigWatcher::reload()` emit `config_changed` event
  verify: `cargo test -p engine-config` + manual: изменение config.toml → UI обновляется без перезапуска

- [ ] 1.4.3 `chat.svelte.ts`: dialogue_turn, answer_token, answer_done listeners
  verify: unit test — streaming accumulation, group by speaker (derived store)

- [ ] 1.4.4 `models.svelte.ts`: lazy load, 5-минутный кэш
  verify: unit test — второй вызов `loadModels()` в течение 5 мин не вызывает invoke

- [ ] 1.4.5 `ui.svelte.ts`: toast queue (max 3 одновременно), modal stack (Escape закрывает верхний)
  verify: unit test — 5 toasts добавлено → показываются 3, остальные в очереди

## Phase 2: Core Components Migration (3-4 weeks)

### 2.1 ModelModal (4 дня)

- [ ] 2.1.1 `ModelModal.svelte` container: open/close, loading skeleton
  verify: клик на пилюлю модели → модалка появляется с fade-in <200ms

- [ ] 2.1.2 `ModelSearch.svelte`: текстовый поиск + capability filters (vision/tools/reasoning)
  verify: query "sonnet" + filter vision=true → показываются только модели с "sonnet" и vision

- [ ] 2.1.3 `FamilySidebar.svelte`: кнопки семейств, sorted alphabetically, active highlight
  verify: клик "Anthropic" → список справа фильтруется, кнопка выделена accent

- [ ] 2.1.4 `ModelList.svelte`: виртуализация (svelte-virtual-list), smooth scroll 60fps
  verify: 417 моделей → в DOM ~15 items, scroll без jank, memory ≤15MB

- [ ] 2.1.5 `ModelCard.svelte`: name, id, badges (ctx, pricing, capabilities icons)
  verify: карточка claude-sonnet-5 показывает "200k", "$3.00/$15.00", vision/tools/reasoning иконки

- [ ] 2.1.6 `ReasoningSlider.svelte`: 5 уровней, live label update, estimated overhead
  verify: slider изменён → label "Высокий", reasoning models показывают ~300 tokens overhead

- [ ] 2.1.7 Model selection: validate via `commands.llmSet()`, toast на ошибку, модалка закрывается на успех
  verify: выбор модели → вызов llmSet, пилюля обновляется, модалка закрывается

- [ ] 2.1.8 Keyboard navigation: Tab cycles, Arrow keys в списке, Enter выбирает, Escape закрывает
  verify: manual test — все hotkeys работают

- [ ] 2.1.9 Accessibility: role=dialog, aria-labelledby, focus trap, screen reader test (NVDA)
  verify: axe-core 0 violations, NVDA объявляет "Model selection dialog"

### 2.2 ChatWindow (5 дней)

- [ ] 2.2.1 `ChatWindow.svelte`: container, autoscroll logic, scroll event handler
  verify: новое сообщение внизу → autoscroll; пользователь наверху → scroll не меняется

- [ ] 2.2.2 `MessageList.svelte`: виртуализация для 100+ сообщений, smooth scroll
  verify: 120 сообщений → в DOM ~10 items, scroll 60fps, memory ≤20MB

- [ ] 2.2.3 `Message.svelte`: speaker avatar, timestamp, content wrapper, collapse logic
  verify: consecutive User messages → grouped под одной аватаркой

- [ ] 2.2.4 `MessageContent.svelte`: memoized markdown parsing (cache WeakMap)
  verify: unit test — одинаковый content рендерится дважды → парсинг только один раз

- [ ] 2.2.5 `CodeBlock.svelte`: syntax highlighting (highlight.js), copy button
  verify: \`\`\`rust code\`\`\` → подсветка синтаксиса, hover → кнопка Copy, клик → clipboard

- [ ] 2.2.6 Streaming answer: partial message с cursor animation, accumulation на answer_token
  verify: Rust emit tokens → UI показывает accumulating text с cursor, answer_done → финализация

- [ ] 2.2.7 Collapse transcripts/operations по `config.chat.collapse_*`
  verify: config.collapse_transcripts=true → transcript сообщения collapsed, expandable

- [ ] 2.2.8 Performance: initial render 50 msg ≤120ms, append 1 msg ≤10ms
  verify: Chrome DevTools Performance профиль — замеры соответствуют targets

- [ ] 2.2.9 Autoscroll re-enable: пользователь скроллит вниз → autoscroll включается снова
  verify: manual test — scroll вверх → autoscroll off, scroll вниз до конца → autoscroll on

### 2.3 ContextModal (1 день)

- [ ] 2.3.1 `ContextModal.svelte`: 3 sliders (recent_window, key_turns_cap, summary_max_tokens)
  verify: изменение slider → live preview estimated tokens обновляется

- [ ] 2.3.2 Apply button: `commands.contextApply()`, toast на успех/ошибку
  verify: клик Apply → config сохраняется, модалка закрывается

- [ ] 2.3.3 Cancel button: закрывает без сохранения, значения rollback
  verify: изменение slider → Cancel → модалка закрывается, config не изменён

### 2.4 HotkeysModal (1 день)

- [ ] 2.4.1 `HotkeysModal.svelte`: список hotkeys, editable inputs
  verify: клик на input → record mode, нажатие Ctrl+Shift+X → сохраняется

- [ ] 2.4.2 Conflict detection: duplicate hotkey → error toast
  verify: ввод уже занятого hotkey → toast "Already assigned to..."

- [ ] 2.4.3 Save button: `commands.hotkeysSet()`, применяется без перезапуска
  verify: изменение hotkey → Save → новый hotkey работает немедленно

### 2.5 Settings Components (2 дня)

- [ ] 2.5.1 `TtsControls.svelte`: toggle button (on/off/auto), mode selection
  verify: клик toggle → `commands.ttsModeSet()`, иконка обновляется

- [ ] 2.5.2 `OpacitySlider.svelte`: live preview (window opacity изменяется на drag)
  verify: drag slider → window opacity меняется в реальном времени

- [ ] 2.5.3 `AccentColorPicker.svelte`: color input, live preview (--color-accent CSS variable)
  verify: выбор цвета → --color-accent обновляется, UI перекрашивается

- [ ] 2.5.4 `ScreenshotControls.svelte`: full/region buttons, Tauri screenshot API
  verify: клик "Region" → cursor crosshair, выделение области → screenshot saved

## Phase 3: Advanced Features (2-3 weeks)

### 3.1 Multi-Provider Switcher (1 день)

- [ ] 3.1.1 `ProviderSwitcher.svelte`: dropdown в ModelModal, список из `config.providers`
  verify: dropdown показывает dslab/openrouter, enabled/disabled badge

- [ ] 3.1.2 Switch provider: `updateConfig('llm.provider', key)`, models refetch
  verify: выбор "openrouter" → config сохраняется, models_list вызывается заново

### 3.2 Responsive Design (2 дня)

- [ ] 3.2.1 Breakpoints: sm (640px), md (768px), lg (1024px) в `tailwind.config.ts`
  verify: tailwind classes `sm:hidden`, `md:flex` компилируются корректно

- [ ] 3.2.2 ModelModal: sidebar overlay на sm (вместо flex row)
  verify: экран <768px → sidebar поверх списка, toggle button открывает/закрывает

- [ ] 3.2.3 Chat: уменьшенные font-size, padding на sm
  verify: экран <640px → text-sm вместо text-base, padding уменьшен

- [ ] 3.2.4 Hotkeys: 1 колонка на sm (вместо 2)
  verify: экран <768px → список hotkeys стекается в 1 колонку

### 3.3 Accessibility Audit & Fixes (3 дня)

- [ ] 3.3.1 Keyboard navigation: Tab focus для всех интерактивных элементов
  verify: Tab циклируется через все кнопки/inputs без пропусков

- [ ] 3.3.2 Arrow keys в списках (ModelList, HotkeysList)
  verify: focus на item → ArrowDown → следующий item, ArrowUp → предыдущий

- [ ] 3.3.3 ARIA: role=dialog, aria-labelledby для всех модалок
  verify: axe-core scan → 0 violations для модалок

- [ ] 3.3.4 ARIA: aria-pressed для toggle buttons (TTS, click-through)
  verify: screen reader объявляет "TTS on, button, pressed"

- [ ] 3.3.5 ARIA: aria-live="polite" для toast notifications
  verify: toast появляется → NVDA объявляет message без прерывания

- [ ] 3.3.6 Focus management: trap focus в модалках, restore после закрытия
  verify: открытие modal → focus внутри, Escape → focus возвращается на триггер

- [ ] 3.3.7 Screen reader test: NVDA (Windows) ручной тест всех критичных flows
  verify: открыть modal → выбрать модель → закрыть — NVDA объявляет все шаги понятно

- [ ] 3.3.8 Lighthouse CI: Performance ≥90, Accessibility ≥95
  verify: CI запускает Lighthouse, PR блокируется если scores падают >5 points

### 3.4 Animations & Micro-interactions (2 дня)

- [ ] 3.4.1 Modal: fade-in + scale animation (transition-base)
  verify: модалка появляется плавно, 250ms cubic-bezier

- [ ] 3.4.2 Toast: slide-in from bottom, auto-dismiss через 3s
  verify: toast появляется снизу с анимацией, исчезает через 3s

- [ ] 3.4.3 Button: ripple effect на клик (Material-style)
  verify: клик на кнопку → ripple анимация от точки клика

- [ ] 3.4.4 Skeleton loaders: models loading state, chat loading
  verify: модалка открыта без данных → skeleton placeholders анимируются

- [ ] 3.4.5 AutoAnimate: список моделей, чат сообщения (FLIP transitions)
  verify: добавление/удаление items → плавная анимация без manual keyframes

## Phase 4: Polish & Testing (1-2 weeks)

### 4.1 Bundle Size Optimization (2 дня)

- [ ] 4.1.1 Code splitting: модалки lazy-load (dynamic imports)
  verify: `npm run build` → отдельные chunks для ModelModal, ContextModal, etc.

- [ ] 4.1.2 Tree-shaking audit: unused exports удалены
  verify: `npx vite-bundle-visualizer` → нет "dead code" в vendor chunks

- [ ] 4.1.3 Marked.js custom build: только нужные features (GFM, breaks)
  verify: marked chunk уменьшился с 15kb до ~8kb

- [ ] 4.1.4 Terser aggressive minification: drop_console, dead_code elimination
  verify: production build → console.log удалены, unreachable code удалён

- [ ] 4.1.5 Final bundle check: total ≤80kb gzip
  verify: `gzip -9 dist/*.js | wc -c` → ≤81920 bytes

### 4.2 Error Boundaries & Fallbacks (1 день)

- [ ] 4.2.1 `ErrorBoundary.svelte`: catch window.error, unhandledrejection
  verify: намеренная ошибка в компоненте → fallback UI с stack trace

- [ ] 4.2.2 Reload button в error boundary
  verify: клик "Reload" → `window.location.reload()`

- [ ] 4.2.3 Graceful degradation: models_list fail → показать cached/empty state
  verify: backend недоступен → toast "Failed to load models", модалка закрывается

### 4.3 Testing Suite (5 дней)

- [ ] 4.3.1 Vitest setup: `vitest.config.ts`, @testing-library/svelte
  verify: `npm run test` запускает пустой suite без ошибок

- [ ] 4.3.2 Unit tests: stores (config sync, chat accumulation, models cache)
  verify: `config.test.ts`, `chat.test.ts`, `models.test.ts` — 20+ assertions

- [ ] 4.3.3 Unit tests: utils (markdown memoization, tauri-bridge helpers)
  verify: `markdown.test.ts` — cache hit/miss scenarios

- [ ] 4.3.4 Component tests: Button (variants, loading, disabled)
  verify: `Button.test.ts` — 10+ test cases, 100% coverage

- [ ] 4.3.5 Component tests: Modal (focus trap, Escape, backdrop click)
  verify: `Modal.test.ts` — axe-core проверка, keyboard events

- [ ] 4.3.6 Component tests: Slider, Input, Select
  verify: по 5+ test cases каждый, aria-* attributes проверены

- [ ] 4.3.7 Playwright setup: `playwright.config.ts`, Tauri integration
  verify: `npx playwright test` запускает пустой suite

- [ ] 4.3.8 E2E test: открыть modal → выбрать модель → verify pill updated
  verify: `e2e/model-selection.spec.ts` — green в CI

- [ ] 4.3.9 E2E test: отправить сообщение → stream answer → verify chat appended
  verify: `e2e/chat-flow.spec.ts` — green

- [ ] 4.3.10 E2E test: изменить context settings → verify config saved
  verify: `e2e/context-settings.spec.ts` — green

- [ ] 4.3.11 Coverage report: 80%+ unit coverage
  verify: `npm run test:coverage` → overall 80%+, critical paths 100%

### 4.4 Performance Benchmarks (1 день)

- [ ] 4.4.1 Benchmark: chat initial render 50 msg → ≤120ms
  verify: автоматический тест с `performance.mark/measure`

- [ ] 4.4.2 Benchmark: chat append 1 msg → ≤10ms
  verify: тест добавляет message, измеряет paint time

- [ ] 4.4.3 Benchmark: model modal render 400 models → smooth scroll 60fps
  verify: Playwright с `page.evaluate(() => performance.now())` измеряет FPS

- [ ] 4.4.4 Benchmark: memory usage 100 messages → ≤20MB
  verify: Chrome DevTools Memory snapshot в automated test

- [ ] 4.4.5 Compare baseline vs new: таблица improvements в `docs/ui/metrics-final.md`
  verify: все targets достигнуты (4.8x faster chat, 42x faster append, etc.)

## Phase 5: Migration Cutover (1 week)

### 5.1 Parallel Run Period (5 дней)

- [ ] 5.1.1 Dual window hotkeys: Ctrl+O (старый), Ctrl+Shift+O (новый)
  verify: оба открываются, работают параллельно, данные shared

- [ ] 5.1.2 Config flag: `[ui] default_version = "next"` swap bindings
  verify: изменение флага → Ctrl+O открывает новый UI

- [ ] 5.1.3 Dogfooding: использовать новый UI как default 5 дней подряд
  verify: GitHub issues с label `ui-next` — собрать feedback

- [ ] 5.1.4 Bug triage: P0 bugs → блокируют cutover, P1/P2 → backlog
  verify: 0 P0 bugs открыто на конец периода

### 5.2 Feature Parity Validation (2 дня)

- [ ] 5.2.1 Checklist: 15 фич старого UI работают в новом
  verify: каждая фича manual test + E2E test green

- [ ] 5.2.2 E2E full suite: 100% критичных paths проходят
  verify: `npm run e2e` → all green

- [ ] 5.2.3 Performance regression check: все targets достигнуты
  verify: `docs/ui/metrics-final.md` заполнен с подтверждением

- [ ] 5.2.4 Accessibility regression check: Lighthouse 95+
  verify: CI Lighthouse report ≥95 для всех страниц

### 5.3 Cutover Execution (1 час)

- [ ] 5.3.1 Atomic rename: `mv ui ui-legacy; mv ui-next/dist ui`
  verify: файловая система — `ui/` содержит новый UI

- [ ] 5.3.2 Update `tauri.conf.json`: `frontendDist = "./ui"`
  verify: файл изменён, diff проверен

- [ ] 5.3.3 Git commit: `git add -A; git commit -m "ui: cutover to Svelte 5 (change 031)"`
  verify: коммит создан, pushed в main

- [ ] 5.3.4 Release notes: changelog с highlights (4.8x faster, a11y 95+, etc.)
  verify: GitHub Release draft готов

- [ ] 5.3.5 Smoke test: `cargo build --release`, запуск → все работает
  verify: приложение стартует, модалка открывается, чат работает

### 5.4 Grace Period & Deprecation (2 недели)

- [ ] 5.4.1 Monitor production: 0 P0 bugs в течение 2 недель
  verify: GitHub issues — нет критичных багов

- [ ] 5.4.2 Git tag: `git tag ui-legacy-final` (backup before deletion)
  verify: tag создан на последнем коммите с legacy UI

- [ ] 5.4.3 Delete legacy: `git rm -rf ui-legacy/; git commit -m "ui: remove legacy after grace period"`
  verify: `ui-legacy/` удалена из репо

## Phase 6: Documentation (3 дня)

- [ ] 6.1 `README.md`: обновлённая секция Development с новыми командами
  verify: `npm run dev`, `npm run build`, `npm test` задокументированы

- [ ] 6.2 `docs/ui/SETUP.md`: установка, dev workflow, troubleshooting
  verify: новый контрибьютор может поднять env за 10 минут по этому гайду

- [ ] 6.3 `docs/ui/COMPONENTS.md`: API reference примитивов (props, events, examples)
  verify: каждый из 8 примитивов задокументирован с code snippets

- [ ] 6.4 `docs/ui/STYLING.md`: design tokens, Tailwind usage, как менять тему
  verify: примеры кастомизации цветов, spacing

- [ ] 6.5 `docs/ui/STATE.md`: stores guide, Tauri bridge patterns
  verify: пример создания нового store с синхронизацией

- [ ] 6.6 `CONTRIBUTING.md`: обновлённый PR workflow (lint, type-check, tests)
  verify: секция UI Development с требованиями к PR

- [ ] 6.7 Архитектурная диаграмма (Mermaid) в README
  verify: граф Tauri → TS API → Stores → Components визуализирован

- [ ] 6.8 Changelog: GitHub Release с полным списком изменений
  verify: release опубликован с тегом `v0.2.0-ui-rewrite`

---

## Success Criteria (все должны быть ✅ перед cutover)

- [ ] Feature parity: 15/15 фич старого UI работают идентично
- [ ] Performance: bundle ≤80kb, chat ≤120ms, append ≤10ms
- [ ] Accessibility: Lighthouse ≥95, manual screen reader pass
- [ ] Tests: 80%+ unit coverage, E2E suite green (100% critical paths)
- [ ] Docs: README, SETUP, COMPONENTS, CONTRIBUTING complete
- [ ] Dogfooding: 5 дней использования без P0 bugs
- [ ] Stakeholder approval: команда подтверждает готовность к cutover

---

## Rollback Plan (если критичный баг после cutover)

1. **Config-based**: пользователь меняет `[ui] default_version = "legacy"` → перезапуск.
2. **Git revert**: `git revert <cutover-commit>; cargo build --release`.
3. **Hotfix binary**: релиз с флагом `--use-legacy-ui`.

---

## Post-Launch Backlog (не блокирует cutover)

- [ ] Command palette (Cmd+K): universal search
- [ ] Themes marketplace: custom accent/fonts
- [ ] Advanced markdown: LaTeX math, mermaid diagrams
- [ ] Chat export: PDF/Markdown
- [ ] Multi-window: отдельные окна для chat/overlay
- [ ] Collaboration: share session URL (real-time sync)
