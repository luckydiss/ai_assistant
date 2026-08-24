# Delta: Config (orchestrator section removed)

## MODIFIED Requirements

### Requirement: Config File Format
Система SHALL читать конфигурацию из TOML с секциями [stt], [llm], [vad], [prompts]. Секция [orchestrator] удалена.

#### Scenario: Валидный config (прежний loads_valid_config, без [orchestrator])
#### Scenario: Несуществующий файл (прежний errors_on_missing_file)
#### Scenario: Невалидный TOML (прежний errors_on_invalid_toml)

### Requirement: Default Values
Система SHALL использовать дефолты для опциональных полей; llm.reasoning_effort SHALL быть опциональным (None по умолчанию).

#### Scenario: Частичный config (прежний applies_defaults)

### Requirement: Config Validation
Система SHALL валидировать значения; проверки min_words удалены вместе с секцией.

#### Scenario: Невалидные пороги (test: validates_thresholds)
- GIVEN vad.silence_ms = 0
- WHEN load
- THEN Err(Validation)

### Requirement: Hot-Reload
(Без изменений: reloads_on_change, keeps_old_on_error сохраняются.)

## ADDED / REMOVED Requirements
(none кроме удалённых полей)
