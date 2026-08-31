# Delta: Config ([context])

## ADDED Requirements

### Requirement: Context Section
[context]: recent_window (default 12), key_turns_cap (default 12), summary_max_tokens (default 300), summary_model (default "" → llm.model).

#### Scenario: Дефолты (test: context_defaults)

#### Scenario: Валидация (test: context_validates)
- GIVEN recent_window=0 → Err(Validation)

## MODIFIED / REMOVED: (none)
