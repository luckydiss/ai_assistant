# Delta: Models Domain (ADDED)

## ADDED Requirements

### Requirement: Model Metadata
Система SHALL предоставлять ModelMetadata struct с полями: id, name, provider, family, context_length, pricing (input/output per 1M tokens), capabilities (chat, vision, tools, reasoning).

#### Scenario: Полные метаданные (test: model_metadata_fields)
- GIVEN OpenRouter response для claude-3.5-sonnet
- WHEN parse
- THEN все поля заполнены

#### Scenario: Деградация к OpenAI-формату (test: metadata_openai_fallback)
- GIVEN OpenAI-format модель (только id) без pricing/architecture
- WHEN parse
- THEN поля принимают дефолты (0 ctx, 0.0 pricing), parsing не падает

### Requirement: Model Provider Trait
Система SHALL определять trait ModelProvider с методами list_models() → Result<Vec<ModelMetadata>> и validate_model(id) → Result<()>.

#### Scenario: Список моделей через trait (test: openrouter_list_models)
- GIVEN mock /models endpoint с rich-метаданными
- WHEN list_models()
- THEN возвращает Vec с метаданными

### Requirement: Capabilities Filtering
Система SHALL фильтровать модели по capabilities: chat=true обязательно; vision/tools/reasoning — опционально.

#### Scenario: Только чатовые (test: filter_chat_only)
- GIVEN список с chat и image моделями
- WHEN filter_chat()
- THEN только chat=true

### Requirement: Family Extraction
Семейство SHALL извлекаться из метаданных провайдера, когда доступно; иначе — единый extractor внутри engine-models (fallback), не в UI.

#### Scenario: Семейство из метаданных (test: family_from_metadata)
- GIVEN модель claude в id/name
- WHEN extract
- THEN family = "Anthropic"

### Requirement: Provider Registry
Система SHALL поддерживать registry провайдеров: [providers.openrouter], [providers.dslab], etc. с полями api_key, base_url, enabled.

#### Scenario: Активный провайдер (test: active_provider)
- GIVEN [llm] provider = "openrouter"
- WHEN get_provider
- THEN возвращает провайдера с base_url openrouter

## MODIFIED / REMOVED: (none)
