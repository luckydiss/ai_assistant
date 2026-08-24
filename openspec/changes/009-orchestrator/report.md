# Report: Orchestrator

## Result

`crates/engine-orchestrator` реализован: `Orchestrator` с командным каналом (turn/manual/fire), дебаунс-триггером, speculative-режимом (>14 слов + "?" → 200мс), last-trigger-wins (abort предыдущего answer), SKIP-скрытием из UI и broadcast-событиями `OrchEvent`. 6/6 тестов, clippy/fmt/release/workspace чисты.

## Deviations from Design

1. **`fire` при пустом фокусе (manual без turn):** дизайн §3 делает `let Some(focus) = ... else { return; }` — это ломало тест `manual_trigger_fires` (спека: manual без turn интервьюера всё равно стреляет в LLM с note). Создаётся пустой `Turn` (Interviewer, пустой текст) как focus-заглушка.

2. **`chrono` в зависимостях:** понадобился для пустого focus-Turn (`Utc::now()`); design §1 его не включает.

3. **`LlmClient::new` возвращает `Result`:** тесты вызывают `.unwrap()` (изменение из 008; дизайн 009 §4 писал без `unwrap`).

4. **`set_summary` не реализован:** дизайн §3 прямо говорит «заглушку НЕ РЕАЛИЗОВЫВАТЬ, если спека не требует»; спека summary-сеттера не требует.

5. **`on_turn` не ждёт 12-turns trim конфликта с summary:** summary в дизайне всегда пустая строка (обновляется только в wiring 010); поведение оставлено как в дизайне.

6. **clippy:** `option_map_unit_fn` → `if let Some(t) = self.trigger.take()`.

## Verified

- `cargo test -p engine-orchestrator` — 6/6 ok (включая `triggers_after_debounce`, `speculative_trigger_fast`, `new_trigger_cancels_previous`, `skip_hidden_from_ui`, `manual_trigger_fires`, `short_turn_ignored`)
- `cargo clippy -p engine-orchestrator --all-targets -- -D warnings` — ok
- `cargo build -p engine-orchestrator --release` — ok
- `cargo build --workspace` — ok
- `cargo fmt -p engine-orchestrator --check` — ok

## Note

STOP Protocol соблюдён: driver-loop lock не блокируется при spawn (fire/on_turn держат lock только на лёгких операциях), mock скопирован из engine-llm/tests без изменений.