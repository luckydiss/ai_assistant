# Tasks: Manual-Only

- [x] 1.1 config.rs: удалить OrchestratorConfig, добавить reasoning_effort, поправить validate по design.md §1
  verify: `cargo build -p engine-config`

- [x] 1.2 Тесты applies_defaults, validates_thresholds обновить
  verify: `cargo test -p engine-config`

- [x] 2.1 builder.rs: новая сигнатура build + user_content по design.md §2
  verify: `cargo build -p engine-context`

- [x] 2.2 Тесты: удалить includes_skip_protocol, добавить builds_without_focus, no_skip_instruction, обновить старые вызовы
  verify: `cargo test -p engine-context`

- [x] 3.1 Удалить skip.rs; AnswerEvent без Skipped; client.rs без Phase/skip_buf; reasoning_effort в body по design.md §3
  verify: `cargo build -p engine-llm`

- [x] 3.2 Тесты: удалить skip-тесты, добавить reasoning_effort_sent; старые streams/cancel/401 зелёные
  verify: `cargo test -p engine-llm`

- [x] 4.1 ПОЛНАЯ ЗАМЕНА orchestrator.rs по design.md §4
  verify: `cargo build -p engine-orchestrator`

- [x] 4.2 Тесты по design.md §5 (удалить авто/SKIP, добавить manual-сценарии)
  verify: `cargo test -p engine-orchestrator`

- [x] 5.1 main.rs wiring по design.md §7
  verify: `cargo build -p desktop`

- [x] 5.2 overlay.js по design.md §7
  verify: manual — речь интервьюера не вызывает generating; «Что сказать» даёт ответ

- [x] 6.1 Заменить config.toml по design.md §6
  verify: `Config::load("config.toml")` ок

- [x] 7.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0

- [x] 8.1 Боевая проверка: 5 минут речи интервьюера → 0 спонтанных ответов; 3 ручных запроса → 3 ответа
  verify: manual check

## STOP Protocol
Если после удаления остались ссылки на Skipped/debounce/min_words — это мёртвый код, удалить; НЕ восстанавливать логику.
Если mock-тесты не компилируются из-за сигнатур — править СТРОГО по design.md §5.
Не добавлять авто-логику обратно ни в каком виде. Сомнение = вопрос человеку.
