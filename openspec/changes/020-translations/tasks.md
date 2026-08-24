# Tasks: Translations

- [ ] 1.1 complete() + translate() в engine-llm по design.md §1
  verify: `cargo build -p engine-llm`

- [ ] 1.2 Тест translate_body_contains_lang (mock с захватом тела)
  verify: `cargo test -p engine-llm translate_body`

- [ ] 2.1 OrchEvent::Translation, TransHandles, set_languages, отмена в fire по design.md §2
  verify: `cargo test -p engine-orchestrator` (старые зелёные)

- [ ] 2.2 Тесты translations_for_extra_langs, max_two_extra_langs, single_lang_no_translation, translation_cancel_on_new_fire
  verify: `cargo test -p engine-orchestrator translation`

- [ ] 3.1 languages в PromptContext + set_languages в pipeline по design.md §2
  verify: `cargo build -p desktop`

- [ ] 3.2 Тест languages_propagate
  verify: `cargo test -p engine-orchestrator languages_propagate`

- [ ] 4.1 Блоки перевода в overlay.js по design.md §3
  verify: manual — перевод под ответом

- [ ] 5.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0

## STOP Protocol
Если Done-ветка форвардера не видит languages — передавать их в spawn форвардера при fire(), НЕ через глобалы. Спросить при затруднении.
