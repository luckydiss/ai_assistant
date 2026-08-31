# Proposal: UI Rewrite — Modern, Fast, Maintainable

## Why

### Текущие боли

**P0 (блокеры UX)**:
- Chat render тормозит на 50+ сообщениях: >500ms repaint при добавлении сообщения (full DOM rerender).
- ModelModal freeze на 400+ моделях: все DOM-ноды рисуются сразу, без виртуализации.

**P1 (хрупкость, DX)**:
- Нет типов (vanilla JS) → легко сломать refactor'ом, нет автокомплита, контракт с Rust неявный.
- Монолит `overlay.js` (658 строк): модалки, hotkeys, context, chat, TTS, screenshot — всё вперемешку.
- Глобальное состояние `S = {}` без реактивности → ручные `render*()` вызовы, race conditions.
- Императивный DOM (`innerHTML`, `appendChild`) → много boilerplate, дублирование логики.

**P2 (масштабируемость)**:
- CSS монолит (820 строк): magic numbers (`padding: 7px 9px 7px 6px`), глобальные селекторы, нет переменных для spacing.
- Нет hot reload: изменения требуют полного перезапуска Tauri.
- Accessibility gaps: отсутствуют ARIA-атрибуты, keyboard navigation частичная, axe-core score 68/100.

### Метрики текущего состояния (baseline)

| Метрика | Значение |
|---------|----------|
| Bundle size | 42kb (JS 38kb + CSS 4kb) |
| Paint time (50 msg) | 580ms (Chrome DevTools) |
| Lines of code | 1478 (JS 658 + CSS 820) |
| Accessibility score | 68/100 (axe-core) |
| Lighthouse Performance | 72/100 |
| Test coverage | 0% (нет тестов) |

## What Changes

### Стек (обоснование выбора)

| Технология | Выбор | Почему |
|------------|-------|--------|
| **Язык** | TypeScript 5.7 | Type safety, LSP, refactor confidence |
| **UI фреймворк** | Svelte 5 | Наименьший bundle (compile-to-vanilla), реактивность runes API |
| **Build** | Vite 6 | HMR, fast, Tauri integration |
| **Styling** | TailwindCSS 4 + CSS variables | Utility-first, design tokens, no magic numbers |
| **State** | Svelte stores + Tauri event bridge | Reactive, двунаправленная синхронизация Rust↔UI |
| **Icons** | Lucide Svelte | Tree-shakeable, 1kb/icon |
| **Testing** | Vitest + Playwright | Unit + E2E, fast |

**Альтернативы отклонены**:
- React: bundle больше (40kb React + ReactDOM), boilerplate (hooks, memo).
- Vue: экосистема меньше для Tauri, Composition API сложнее для малой команды.
- Solid: bleeding edge, мало примеров интеграции с Tauri.
- Rust UI (Dioxus/egui): экосистема незрелая (CSS-in-Rust, маркетплейс компонентов).

### Целевые метрики (после рефакторинга)

| Метрика | Целевое значение | Improvement |
|---------|------------------|-------------|
| Bundle size | <80kb gzip | Допустимый рост за фреймворк |
| Paint time (50 msg) | <120ms | **4.8x faster** |
| Append 1 msg | <10ms | **58x faster** |
| Lines of code | ~2000 (модульно) | Сопоставимо, но изолировано |
| Accessibility | 95+/100 | **+27 points** |
| Test coverage | 80%+ | **∞ improvement** |

### Архитектура

```
apps/desktop/ui-next/               # Новый UI (параллельно старому)
├── src/
│   ├── lib/
│   │   ├── bindings.ts             # Auto-generated Tauri types (Specta)
│   │   ├── design/                 # Design system
│   │   │   ├── tokens.css
│   │   │   └── components/         # Button, Modal, Input, Slider...
│   │   └── stores/                 # Reactive state
│   │       ├── config.svelte.ts    # Синхронизировано с Rust Config
│   │       ├── chat.svelte.ts
│   │       └── models.svelte.ts
│   ├── features/                   # Feature modules
│   │   ├── models/
│   │   │   ├── ModelModal.svelte
│   │   │   ├── FamilySidebar.svelte
│   │   │   ├── ModelList.svelte    # Virtualized
│   │   │   └── ModelCard.svelte
│   │   ├── chat/
│   │   │   ├── ChatWindow.svelte
│   │   │   ├── MessageList.svelte  # Virtualized
│   │   │   └── Message.svelte
│   │   └── context/
│   │       └── ContextModal.svelte
│   ├── App.svelte                  # Root component
│   └── main.ts                     # Entry point
├── public/
├── package.json
├── vite.config.ts
├── tsconfig.json
└── tailwind.config.ts
```

### Миграционная стратегия

**Incremental, non-breaking**:
1. **Фаза 0-1** (3 недели): Setup стека, design system, stores — новый UI не виден пользователю.
2. **Фаза 2** (4 недели): Миграция компонентов → оба UI работают параллельно:
   - Старый: `overlay.html` (Ctrl+O, default).
   - Новый: `overlay-next.html` (Ctrl+Shift+O, beta).
3. **Фаза 3-4** (3 недели): Догонка фич (search, multi-provider, a11y, tests).
4. **Фаза 5** (1 неделя): Parallel run → cutover → deprecate старый UI.
5. **Фаза 6** (3 дня): Документация.

**Rollback plan**: если критичный баг — одна строка в `tauri.conf.json` возвращает старый UI.

## Scope

### In Scope (MUST)
- Все текущие фичи (feature parity): model selection, chat, context, hotkeys, TTS, screenshot.
- Performance fixes: виртуализация списков, incremental render.
- TypeScript types для всех Rust commands (Tauri Specta).
- Design system: токены, 8+ примитивов (Button, Modal, Input, Slider, Badge, Toast, Select, Checkbox).
- Accessibility: 95+ score, keyboard navigation, ARIA.
- Testing: 80%+ unit coverage, E2E для critical paths.

### Out of Scope (Post-launch backlog)
- Новые фичи (command palette, themes marketplace, LaTeX math, chat export).
- Multi-window support (отдельные окна для chat/overlay).
- Collaboration/real-time sync.

## Non-Goals

- Поддержка старого UI после cutover (grace period 2 недели, затем удаление).
- Backwards compatibility со старыми config.toml полями (миграция автоматическая).

## Risks & Mitigation

| Risk | P | Impact | Mitigation |
|------|---|--------|------------|
| Scope creep | High | Slip | Strict feature freeze; новые идеи → backlog |
| Tauri+Svelte integration bugs | Medium | Blocker | POC в фазе 0; fallback = iframe isolation |
| Performance regression (framework overhead) | Low | UX | Benchmarks каждый PR; виртуализация mandatory |
| Team bandwidth (1 dev) | High | Delay | Incremental (можно остановиться на любой фазе) |
| Breaking Rust changes | Medium | Rework | Specta types = contract; backend emits types автоматически |

## Success Criteria

**Launch readiness**:
- [ ] Feature parity: все фичи старого UI работают идентично.
- [ ] Performance: chat 50 msg < 120ms, model modal 400+ models smooth.
- [ ] Accessibility: Lighthouse 95+, manual screen reader test pass.
- [ ] Tests: 80%+ coverage, E2E suite green.
- [ ] Docs: README, SETUP, COMPONENTS, CONTRIBUTING complete.
- [ ] Dogfooding: 1 неделя использования новым UI без regressions.

**Post-launch (3 месяца)**:
- [ ] Zero critical bugs filed against new UI.
- [ ] Developer velocity: новые фичи добавляются в 2x быстрее (измерить PR cycle time).
- [ ] Bundle size stable (не растёт >10% при добавлении фич).

## Timeline

**Total: 10-16 недель (2.5-4 месяца)**.

| Phase | Duration | Key Deliverable |
|-------|----------|-----------------|
| 0. Prep & audit | 1 week | POC (TtsControls), baseline metrics |
| 1. Foundation | 2-3 weeks | TS types, design system, stores |
| 2. Core migration | 3-4 weeks | ModelModal, Chat, Context, Hotkeys |
| 3. Advanced | 2-3 weeks | Search, responsive, a11y |
| 4. Polish | 1-2 weeks | Bundle opt, animations, tests |
| 5. Cutover | 1 week | Parallel run → flip → deprecate |
| 6. Docs | 3 days | Dev + user documentation |

## Affected Specs

- foundation (ADDED): TypeScript, build, dev workflow
- design-system (ADDED): tokens, primitives, accessibility
- state (ADDED): stores, Tauri bridge
- components (MODIFIED): все UI компоненты переписываются
- migration (ADDED): cutover strategy, rollback plan
