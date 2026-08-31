# Tasks: Long-Context Memory

- [x] 1.1 ContextSection в config по design.md §1 + тесты context_defaults, context_validates
  verify: `cargo test -p engine-config`

- [x] 2.1 ContextInput + новый build/user_content по design.md §2; обновить старые вызовы/тесты
  verify: тесты builds_all_layers, skips_empty_layers, budget_safety, context_input_fields

- [x] 3.1 is_key_turn + key_turns + drain + summarize + Cmd::SummaryDone по design.md §3
  verify: тесты key_question_detected, short_not_key, key_turns_cap, recent_window_drain, summary_updates

- [x] 4.1 fire() собирает ContextInput по design.md §4
  verify: `cargo build -p engine-orchestrator`

- [ ] 5.1 Боевая проверка: 40+ реплик → в теле запроса присутствуют "Резюме всей сессии" и "Ключевые моменты", recent ≤ 12
  verify: manual + лог тела запроса

- [x] 6.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0

## STOP Protocol
Если LlmClient::complete сигнатура отличается — свериться с 021/026, НЕ менять стриминг.
Если drain срабатывает слишком часто/редко — тюнить recent_window в конфиге, не логику.
