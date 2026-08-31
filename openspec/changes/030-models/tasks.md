# Tasks: Unified Model Providers

- [x] 1.1 engine-models crate + async-trait в workspace deps
  verify: `cargo build -p engine-models`

- [x] 2.1 metadata.rs: ModelMetadata/Pricing/Capabilities (design §2)
  verify: `cargo build -p engine-models`

- [x] 3.1 provider.rs: trait ModelProvider (design §3)
  verify: `cargo build -p engine-models`

- [x] 4.1 catalog.rs: OpenAiCompatCatalog — rich/plain parsing, filter, family
  verify: тесты model_metadata_fields, metadata_openai_fallback,
          filter_chat_only, family_from_metadata

- [x] 5.1 catalog.rs: validate_model (unknown → Err; http-down → Ok+warn)
  verify: тест validate_on_mock

- [x] 6.1 config: ProviderConfig/providers/get_provider()/normalize() (design §5)
  verify: тесты legacy_migration_nonbreaking, openrouter_defaults,
          validates_provider_exists

- [x] 7.1 engine-llm: with_catalog + validate_model (design §6)
  verify: `cargo test -p engine-llm` (temperature_retry не сломан)

- [x] 7.2 commands.rs: models_list → Vec<ModelMetadata>; llm_set с валидацией (design §7)
  verify: `cargo build -p desktop`

- [x] 8.1 UI rewrite: убрать familyOf/prettyName/цвета-хардкод; метаданные в карточках (design §8)
  verify: manual — модалка группирует по семействам из бэкенда, видны ctx/pricing/badges

- [ ] 9.1 Боевая проверка: модалка на dslab.tech показывает модели; выбор модели работает;
      config.toml сохраняет provider/model
  verify: manual

- [x] 10.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0

## STOP Protocol
- Если миграция ломает существующие конфиги — НЕ делать авто-замену credentials:
  легаси-endpoint остаётся провайдером как есть (реализовано так и есть).
- Если OpenRouter /models отдаёт неизвестную структуру — graceful degradation к
  plain OpenAI-формату (id-only), не ломать парсинг.
