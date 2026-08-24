# Tasks: Bootstrap

## Phase 1: Root Configuration

- [ ] 1.1 Создать `Cargo.toml` в корне с workspace определением из design.md §1
  verify: `cargo metadata --format-version 1` не падает
  
- [ ] 1.2 Создать `.gitignore` в корне из design.md §5
  verify: `git status` показывает только новые файлы

- [ ] 1.3 Создать `rustfmt.toml` в корне из design.md §6
  verify: `cargo fmt --check` проходит на пустом workspace

- [ ] 1.4 Создать `clippy.toml` в корне из design.md §7
  verify: файл существует

## Phase 2: Engine Crates Shells

- [ ] 2.1 Создать `crates/engine-config/` с Cargo.toml и src/lib.rs по шаблону design.md §2
  verify: `cargo build -p engine-config` проходит

- [ ] 2.2 Создать `crates/engine-audio/` по шаблону
  verify: `cargo build -p engine-audio` проходит

- [ ] 2.3 Создать `crates/engine-vad/` по шаблону
  verify: `cargo build -p engine-vad` проходит

- [ ] 2.4 Создать `crates/engine-stt/` по шаблону
  verify: `cargo build -p engine-stt` проходит

- [ ] 2.5 Создать `crates/engine-dialogue/` по шаблону
  verify: `cargo build -p engine-dialogue` проходит

- [ ] 2.6 Создать `crates/engine-orchestrator/` по шаблону
  verify: `cargo build -p engine-orchestrator` проходит

- [ ] 2.7 Создать `crates/engine-context/` по шаблону
  verify: `cargo build -p engine-context` проходит

- [ ] 2.8 Создать `crates/engine-llm/` по шаблону
  verify: `cargo build -p engine-llm` проходит

- [ ] 2.9 Создать `crates/engine-store/` по шаблону
  verify: `cargo build -p engine-store` проходит

- [ ] 2.10 Создать `crates/engine-ipc/` по шаблону
  verify: `cargo build -p engine-ipc` проходит

## Phase 3: Desktop App

- [ ] 3.1 Создать `apps/desktop/Cargo.toml` из design.md §3
  verify: `cargo build -p desktop` проходит

- [ ] 3.2 Создать `apps/desktop/src/main.rs` из design.md §3
  verify: `cargo build -p desktop` проходит

- [ ] 3.3 Создать `apps/desktop/build.rs` из design.md §3
  verify: `cargo build -p desktop` проходит

- [ ] 3.4 Создать `apps/desktop/src-tauri/tauri.conf.json` из design.md §3
  verify: `cargo build -p desktop` проходит

## Phase 4: CI

- [ ] 4.1 Создать `.github/workflows/ci.yml` из design.md §4
  verify: файл валидный YAML

## Phase 5: Validation

- [ ] 5.1 Запустить `cargo fmt --all`
  verify: выход 0

- [ ] 5.2 Запустить `cargo clippy --workspace --all-targets -- -D warnings`
  verify: выход 0

- [ ] 5.3 Запустить `cargo build --workspace`
  verify: выход 0

- [ ] 5.4 Запустить `cargo test --workspace`
  verify: выход 0 (тестов нет, но команда проходит)

## STOP Protocol

Если:
- `cargo build` падает с ошибкой про зависимость → проверить Cargo.toml точное совпадение с design.md
- Crate не находится в workspace → проверить members в root Cargo.toml
- Tauri build падает → проверить tauri.conf.json точно совпадает

Не пытаться "починить" соседние файлы. Остановиться и спросить.
