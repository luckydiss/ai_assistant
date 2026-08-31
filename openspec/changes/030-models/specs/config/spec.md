# Delta: Config (providers section)

## MODIFIED Requirements

### Requirement: Config File Format
[llm]: provider, model, reasoning_effort, max_tokens, temperature (+ легаси base_url/api_key для обратной совместимости).
[providers.<name>]: api_key, base_url (default "https://openrouter.ai/api/v1" для openrouter), enabled.

#### Scenario: OpenRouter дефолт (test: openrouter_defaults)
- GIVEN config без providers и без легаси-credentials
- WHEN get_provider при llm.provider = "openrouter"
- THEN base_url = "https://openrouter.ai/api/v1"

#### Scenario: Небомящая миграция dslab.tech (test: legacy_migration_nonbreaking)
- GIVEN старый config с llm.base_url = "https://api.dslab.tech/v1" и api_key
- WHEN load
- THEN синтезируется [providers.dslab] с ТЕМИ ЖЕ credentials,
  llm.provider = "dslab", рабочая авторизация не ломается
  (см. STOP Protocol: авто-замена на openrouter.ai запрещена)

### Requirement: Provider Validation
Система SHALL валидировать, что provider из [llm] существует в [providers.*], когда секция [providers] присутствует. Пустой provider и отсутствие секции — валидны (легаси-режим).

#### Scenario: Несуществующий провайдер (test: validates_provider_exists)
- GIVEN [llm] provider = "unknown" и непустой [providers]
- WHEN validate
- THEN Err(Validation)

## ADDED / REMOVED: (иных нет)
