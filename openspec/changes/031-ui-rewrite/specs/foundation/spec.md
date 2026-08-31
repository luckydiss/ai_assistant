# Delta: Foundation (ADDED)

## ADDED Requirements

### Requirement: TypeScript Build System
Система SHALL компилировать TypeScript 5.7+ в JavaScript с target ES2020, поддерживая ES modules и strict mode.

#### Scenario: Успешная компиляция (test: tsc_compiles_without_errors)
- GIVEN валидный TypeScript код в `ui-next/src/`
- WHEN `npm run build`
- THEN выход 0, артефакты в `ui-next/dist/`, size.json содержит bundle metrics

#### Scenario: Type errors блокируют build (test: tsc_fails_on_type_error)
- GIVEN TypeScript код с ошибкой типа (например, `invoke<string>('models_list')` вместо `ModelMetadata[]`)
- WHEN `npm run build`
- THEN выход ≠0, stderr содержит "Type 'string' is not assignable"

### Requirement: Tauri Types Generation (Specta)
Система SHALL автоматически генерировать TypeScript типы из Rust команд при каждом `cargo build`.

#### Scenario: Типы синхронизированы (test: specta_generates_types)
- GIVEN Rust команда аннотирована `#[specta::specta]` и `#[derive(Type)]` для структур
- WHEN `cargo build -p desktop`
- THEN `ui-next/src/bindings.ts` содержит экспортированные типы и функции-обёртки

#### Scenario: Изменение Rust сигнатуры обновляет types (test: specta_updates_on_change)
- GIVEN команда `models_list` изменила возвращаемый тип `Vec<String>` → `Vec<ModelMetadata>`
- WHEN `cargo build -p desktop`
- THEN `bindings.ts` содержит `modelsList(): Promise<ModelMetadata[]>`
- AND TypeScript код с устаревшим типом не компилируется

### Requirement: Vite Development Server
Система SHALL запускать dev server с HMR на `http://localhost:5173`.

#### Scenario: HMR работает (test: manual_hmr_updates)
- GIVEN запущен `npm run dev`
- WHEN изменён файл `Button.svelte` (props: добавлен `size`)
- THEN браузер обновляется <200ms без full reload
- AND состояние компонентов сохраняется (модалка остаётся открытой)

#### Scenario: Tauri invoke работает в dev mode (test: dev_server_invokes_rust)
- GIVEN запущены `npm run dev` И Tauri app
- WHEN UI вызывает `commands.modelsList()`
- THEN запрос идёт к Rust backend, возвращает данные

### Requirement: Build Artifacts Integration
Tauri build script SHALL читать `ui-next/dist/` как `frontendDist`, если директория существует.

#### Scenario: Production build использует новый UI (test: tauri_serves_new_ui)
- GIVEN `ui-next/dist/` содержит скомпилированные assets
- WHEN `cargo build -p desktop --release`
- THEN `target/release/desktop.exe` сервит `ui-next/dist/index.html` как overlay
- AND старый `ui/overlay.html` не загружается

#### Scenario: Fallback на старый UI (test: tauri_fallback_old_ui)
- GIVEN `ui-next/dist/` не существует (удалена)
- WHEN `cargo build -p desktop`
- THEN Tauri читает `ui/` как frontendDist (старый UI)

### Requirement: ESLint + Prettier
Система SHALL проверять код на стиль и потенциальные ошибки.

#### Scenario: Lint pass (test: eslint_passes)
- GIVEN код соответствует правилам `.eslintrc.json`
- WHEN `npm run lint`
- THEN выход 0

#### Scenario: Lint fail блокирует commit (test: eslint_blocks_commit)
- GIVEN код с unused variable
- WHEN `git commit` (pre-commit hook)
- THEN commit rejected, stderr содержит ESLint ошибки

### Requirement: Bundle Size Budget
Build SHALL фейлиться, если gzipped bundle > 80kb.

#### Scenario: Bundle в пределах бюджета (test: bundle_size_within_budget)
- GIVEN финальный build
- WHEN проверяется `dist/**/*.js` gzipped size
- THEN total ≤ 80kb

#### Scenario: Bundle превышает бюджет (test: bundle_size_exceeds_budget)
- GIVEN добавлена тяжёлая библиотека (например, moment.js 70kb)
- WHEN `npm run build`
- THEN warning в stdout: "Bundle size 95kb exceeds budget 80kb"

## MODIFIED Requirements
(нет — foundation новый слой)

## REMOVED Requirements
(нет)
