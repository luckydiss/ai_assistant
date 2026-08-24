# Delta: Config

## ADDED Requirements

### Requirement: Config File Format
Система SHALL читать конфигурацию из TOML файла с секциями `[stt]`, `[llm]`, `[vad]`, `[orchestrator]`, `[prompts]`.

#### Scenario: Валидный config файл (test: loads_valid_config)
- GIVEN файл config.toml с валидным TOML
- WHEN вызван `Config::load("config.toml")`
- THEN возвращается `Ok(Config)` с заполненными полями

#### Scenario: Несуществующий файл (test: errors_on_missing_file)
- GIVEN файл не существует
- WHEN вызван `Config::load("missing.toml")`
- THEN возвращается `Err(ConfigError::Io)`

#### Scenario: Невалидный TOML (test: errors_on_invalid_toml)
- GIVEN файл с синтаксической ошибкой TOML
- WHEN вызван `Config::load("config.toml")`
- THEN возвращается `Err(ConfigError::Parse)`

### Requirement: Default Values
Система SHALL использовать default значения для всех опциональных полей если они не заданы в файле.

#### Scenario: Частичный config (test: applies_defaults)
- GIVEN config.toml с только `[stt]` секцией
- WHEN загружен config
- THEN `[vad]`, `[llm]`, `[orchestrator]` имеют default значения

### Requirement: Hot-Reload
Система SHALL автоматически перезагружать config при изменении файла.

#### Scenario: Изменение файла (test: reloads_on_change)
- GIVEN config загружен и watcher запущен
- WHEN файл config.toml изменён
- THEN новый config доступен через 100мс
- AND событие `ConfigEvent::Changed` отправлено подписчикам

#### Scenario: Ошибка при reload (test: keeps_old_on_error)
- GIVEN config загружен и watcher запущен
- WHEN файл изменён на невалидный TOML
- THEN старый config остаётся активным
- AND событие `ConfigEvent::Error` отправлено

### Requirement: Config Validation
Система SHALL валидировать значения config при загрузке.

#### Scenario: Невалидные пороги (test: validates_thresholds)
- GIVEN config с `vad.silence_ms = -1`
- WHEN загружен config
- THEN возвращается `Err(ConfigError::Validation)`

## MODIFIED Requirements

(none)

## REMOVED Requirements

(none)
