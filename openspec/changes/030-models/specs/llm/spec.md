# Delta: LLM (provider abstraction)

## MODIFIED Requirements

### Requirement: LLM Client
LlmClient SHALL принимать опциональный каталог моделей (Arc<dyn ModelProvider>) для валидации; HTTP-запросы идут на base_url/api_key провайдера. Все существующие сигнатуры вызовов (orchestrator, tests) сохраняют совместимость.

#### Scenario: Provider-agnostic (test: llm_uses_provider)
- GIVEN LlmClient с произвольным OpenAI-compatible endpoint
- WHEN complete()
- THEN запрос идёт на endpoint провайдера

### Requirement: Temperature Handling
Клиент SHALL продолжать retry без temperature при 400 "compatibility policy" (для GPT-5.6-семейства), но это fallback, а не основная логика.

#### Scenario: Retry без temperature (test: temperature_retry)
- GIVEN 400 response с "compatibility policy"
- WHEN retry
- THEN запрос без temperature успешен

## ADDED Requirements

### Requirement: Model Validation
При смене модели (llm_set) клиент SHALL валидировать id через каталог провайдера — если модель не существует, возвращать Err до переключения живого клиента.

#### Scenario: Валидация модели при переключении (test: validate_on_set)
- GIVEN несуществующая модель
- WHEN llm_set
- THEN Err, живой клиент не меняется

## REMOVED Requirements
(Стоп-слова фильтрация переносится из commands.rs внутрь engine-models как capability-heuristic)
