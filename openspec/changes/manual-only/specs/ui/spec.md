# Delta: UI (no skipped, on-demand hint)

## MODIFIED Requirements

### Requirement: Status Indicator
Оверлей SHALL показывать статусы listening/generating/error; в listening SHALL показывать подсказку с хоткеем ручного запроса; обработка события answer_skipped удалена.

#### Scenario: Статус и подсказка (test: manual_status_hint)
- GIVEN запущенное приложение
- WHEN статус listening
- THEN видна подсказка «Что сказать — Ctrl+Shift+Space» (manual)

#### Scenario: Речь не вызывает генерацию (test: manual_no_spontaneous)
- WHEN интервьюер говорит
- THEN статус остаётся listening, generating появляется только после действия пользователя (manual)

## ADDED / REMOVED
(none)
