# Proposal: Bootstrap Workspace

## Why
Нужна базовая структура workspace для разработки Rust-движка с Tauri UI.

## What Changes
- Создание workspace с пустыми crates-оболочками
- Настройка CI (GitHub Actions)
- Базовые lints и conventions

## Scope
- Cargo workspace configuration
- Empty crate shells with Cargo.toml
- GitHub Actions workflow
- Clippy + fmt configuration

## Non-Goals
- Никакой бизнес-логики
- Никаких зависимостей кроме project.md matrix
- Никаких тестов (только структура)

## Affected Specs
- None (infrastructure only)
