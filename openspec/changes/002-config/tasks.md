# Tasks: Config System

## Phase 1: Dependencies

- [ ] 1.1 Обновить `crates/engine-config/Cargo.toml` добавить зависимости из design.md §1
  verify: `cargo build -p engine-config` проходит

## Phase 2: Error Types

- [ ] 2.1 Создать `crates/engine-config/src/error.rs` из design.md §1
  verify: `cargo build -p engine-config` проходит

## Phase 3: Config Struct

- [ ] 3.1 Создать `crates/engine-config/src/config.rs` с Config и вложенными structs из design.md §1
  verify: `cargo build -p engine-config` проходит

- [ ] 3.2 Реализовать `Config::load()` метод из design.md §1
  verify: `cargo build -p engine-config` проходит

- [ ] 3.3 Реализовать `validate()` метод из design.md §1
  verify: `cargo build -p engine-config` проходит

## Phase 4: Watcher

- [ ] 4.1 Создать `crates/engine-config/src/watcher.rs` с ConfigEvent enum из design.md §1
  verify: `cargo build -p engine-config` проходит

- [ ] 4.2 Реализовать `ConfigWatcher::start()` метод из design.md §1
  verify: `cargo build -p engine-config` проходит

## Phase 5: Public API

- [ ] 5.1 Обновить `crates/engine-config/src/lib.rs` с pub use из design.md §1
  verify: `cargo build -p engine-config` проходит

## Phase 6: Tests

- [ ] 6.1 Создать `crates/engine-config/tests/config_tests.rs`
  verify: файл создан

- [ ] 6.2 Тест `loads_valid_config` (scenario из specs)
  verify: `cargo test -p engine-config loads_valid_config` проходит

- [ ] 6.3 Тест `errors_on_missing_file` (scenario из specs)
  verify: `cargo test -p engine-config errors_on_missing_file` проходит

- [ ] 6.4 Тест `errors_on_invalid_toml` (scenario из specs)
  verify: `cargo test -p engine-config errors_on_invalid_toml` проходит

- [ ] 6.5 Тест `applies_defaults` (scenario из specs)
  verify: `cargo test -p engine-config applies_defaults` проходит

- [ ] 6.6 Тест `validates_thresholds` (scenario из specs)
  verify: `cargo test -p engine-config validates_thresholds` проходит

- [ ] 6.7 Тест `reloads_on_change` (scenario из specs) - async тест с tokio
  verify: `cargo test -p engine-config reloads_on_change` проходит

- [ ] 6.8 Тест `keeps_old_on_error` (scenario из specs) - async тест
  verify: `cargo test -p engine-config keeps_old_on_error` проходит

## Phase 7: Example Config

- [ ] 7.1 Создать `config.toml` в корне из design.md §2
  verify: `Config::load("config.toml")` работает (ручная проверка)

## Phase 8: Validation

- [ ] 8.1 Запустить `cargo clippy -p engine-config --all-targets -- -D warnings`
  verify: выход 0

- [ ] 8.2 Запустить `cargo test -p engine-config`
  verify: все тесты проходят

- [ ] 8.3 Запустить `cargo build -p engine-config --release`
  verify: выход 0

## STOP Protocol

Если:
- `toml::from_str` не парсит валидный TOML → проверить точное совпадение struct полей с TOML ключами
- `notify` watcher не срабатывает → проверить path.exists() перед watch
- Broadcast channel не получает события → проверить что sender.clone() передан в closure

Не пытаться "улучшить" watcher или добавить новые фичи. Остановиться и спросить.
