# Proposal: Configuration System

## Why
Нужна система конфигурации с hot-reload для промптов, порогов и настроек моделей.

## What Changes
- TOML-based config file
- Hot-reload через notify crate
- Config struct с валидацией
- Default values для всех полей

## Scope
- Config file format (TOML)
- Config loading и watching
- Public API: `Config::load()`, `Config::watch()`
- Error types для config ошибок

## Non-Goals
- GUI для редактирования конфига (только файл)
- Remote config (только локальный файл)
- Encryption чувствительных данных

## Affected Specs
- config
