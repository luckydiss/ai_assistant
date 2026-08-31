# Delta: Migration Strategy (ADDED)

## ADDED Requirements

### Requirement: Parallel UI Deployment
Система SHALL поддерживать одновременную работу старого и нового UI в течение cutover фазы.

#### Scenario: Dual window access (test: dual_ui_accessible)
- GIVEN приложение запущено с обоими UI скомпилированными
- WHEN пользователь нажимает Ctrl+O
- THEN открывается старый UI (`ui/overlay.html`)
- WHEN пользователь нажимает Ctrl+Shift+O
- THEN открывается новый UI (`ui-next/dist/index.html`)
- AND оба окна могут работать одновременно, данные shared (Rust backend один)

#### Scenario: Config flag для выбора default (test: config_default_ui_flag)
- GIVEN `config.toml` содержит `[ui] default_version = "next"`
- WHEN приложение стартует
- THEN Ctrl+O открывает новый UI, Ctrl+Alt+O — старый (swap bindings)

### Requirement: Feature Parity Validation
Cutover SHALL произойти только после подтверждения полной feature parity.

#### Scenario: Checklist completion (test: feature_parity_checklist_complete)
- GIVEN checklist в `tasks.md` с 15 фичами
- WHEN все 15 отмечены ✅
- THEN E2E test suite запускается
- AND все тесты green → cutover разрешён

#### Scenario: Manual validation (test: manual_dogfooding_1_week)
- GIVEN новый UI используется как default в течение 1 недели
- WHEN собираются feedback (GitHub issues с label `ui-next`)
- THEN если критичных багов 0 → cutover разрешён
- ELSE если есть P0 баги → откат, фикс, повтор недели

### Requirement: Rollback Plan
Система SHALL поддерживать мгновенный rollback на старый UI при критичных багах.

#### Scenario: Rollback через config (test: rollback_via_config)
- GIVEN новый UI активен по умолчанию
- WHEN обнаружен критичный баг (например, chat не рендерится)
- THEN пользователь изменяет `config.toml`: `[ui] default_version = "legacy"`
- AND перезапускает приложение
- THEN старый UI загружается, работает идентично как до cutover

#### Scenario: Rollback через hotfix binary (test: rollback_via_binary)
- GIVEN распространён бинарник с новым UI
- WHEN масс-репорты о баге
- THEN выпускается hotfix версия с флагом `--use-legacy-ui`
- AND пользователи запускают `desktop.exe --use-legacy-ui`
- THEN приложение использует старый UI без изменения конфига

### Requirement: Cutover Execution
Cutover SHALL произойти через atomic rename + git commit.

#### Scenario: Cutover steps (test: cutover_atomic)
- GIVEN новый UI прошёл все валидации
- WHEN запускается cutover script:
  ```bash
  mv ui ui-legacy
  mv ui-next/dist ui
  # update tauri.conf.json: frontendDist = "./ui"
  git add -A
  git commit -m "ui: cutover to Svelte 5 (change 031)"
  ```
- THEN следующий build использует новый UI
- AND старый UI сохранён в `ui-legacy/` (grace period 2 недели)

### Requirement: Deprecation Timeline
Старый UI SHALL удаляться через 2 недели после успешного cutover.

#### Scenario: Grace period (test: grace_period_2_weeks)
- GIVEN cutover произошёл 1 января
- WHEN текущая дата < 15 января
- THEN `ui-legacy/` существует в репо
- AND документация упоминает rollback процедуру

#### Scenario: Deprecation (test: legacy_ui_removed)
- GIVEN прошло 2 недели после cutover без критичных багов
- WHEN запускается cleanup:
  ```bash
  git tag ui-legacy-final $(git rev-parse HEAD)
  git rm -rf ui-legacy/
  git commit -m "ui: remove legacy UI after 2-week grace period"
  ```
- THEN `ui-legacy/` удалена из репо
- AND git tag сохраняет последний коммит с legacy UI для истории

### Requirement: Performance Regression Detection
CI SHALL мониторить performance metrics после cutover.

#### Scenario: Bundle size regression (test: ci_bundle_size_check)
- GIVEN каждый PR проверяется на bundle size
- WHEN новый код добавлен
- THEN если bundle вырос >5% → CI fail, PR блокируется
- AND отчёт показывает: "Bundle 85kb exceeds budget 80kb (+6.25%)"

#### Scenario: Runtime performance regression (test: ci_lighthouse_check)
- GIVEN каждый PR запускает Lighthouse CI
- WHEN новый код добавлен
- THEN если Performance score упал >5 points → warning в PR
- AND если упал >10 points → CI fail

### Requirement: Migration Documentation
Cutover SHALL сопровождаться обновлённой документацией для разработчиков.

#### Scenario: README updated (test: readme_reflects_new_stack)
- GIVEN cutover завершён
- WHEN `README.md` прочитан
- THEN секция "Development" содержит:
  - Новые команды: `npm run dev`, `npm run build`, `npm test`
  - Архитектурная диаграмма Svelte app
  - Ссылки на `docs/ui/SETUP.md`, `docs/ui/COMPONENTS.md`

#### Scenario: CONTRIBUTING updated (test: contributing_has_new_workflow)
- GIVEN cutover завершён
- WHEN `CONTRIBUTING.md` прочитан
- THEN содержит:
  - Требования к PR: `npm run lint`, `npm run type-check`, `npm test` проходят
  - Как добавить компонент (использовать design system primitives)
  - Как писать тесты (Vitest для unit, Playwright для E2E)

## MODIFIED Requirements
(нет)

## REMOVED Requirements
(нет)
