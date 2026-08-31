# Proposal: Unified Model Providers Architecture

## Why
Текущая система выбора модели — набор костылей:
- Фильтрация по списку стоп-слов (tts, image, embed...) — хрупко, ломается при появлении новых моделей
- familyOf через regex-цепочку — хрупко, требует дописывания при каждом новом семействе
- Хардкод цветов семейств в UI
- Нет абстракции провайдера — всё завязано на один endpoint
- Нет метаданных моделей (context_length, pricing, capabilities)

Переходим на OpenRouter как основной провайдер (агрегатор 200+ моделей от всех провайдеров), строим расширяемую архитектуру с capabilities-driven фильтрацией.

## What Changes
- Новый crate engine-models: trait ModelProvider + реализация OpenRouter
- Model metadata: id, name, family, context_length, pricing, capabilities
- Capabilities-driven фильтрация (chat, vision, tools) вместо стоп-слов
- Семейства из метаданных (architecture.family) вместо regex
- Config: [providers.openrouter] + [llm] provider
- UI: группировка по метаданным, без хардкода
- Migration: dslab.tech → OpenRouter (НЕ breaking: см. STOP Protocol)

## Scope
- engine-models crate
- Интеграция с engine-llm (provider abstraction)
- UI модалка выбора модели (переписать)
- Config schema update

## Non-Goals
- Fallback на несколько провайдеров одновременно (один активный)
- Кэширование списка моделей (запрос на каждое открытие модалки)
- Streaming через WebSocket (OpenRouter использует SSE)

## Affected Specs
- models (ADDED), llm (MODIFIED), config (MODIFIED), ui (MODIFIED)

## Deviations from original design (decided during implementation)
1. **Миграция dslab.tech** — по STOP Protocol сделана небомящей: легаси-
   credentials сохраняются как запись `[providers.<host>]`, ничего не заменяется.
   Причина: ключ dslab.tech не действителен на openrouter.ai — автоматическая
   подмена сломала бы рабочую авторизацию. OpenRouter подключается вручную.
2. **Generic OpenAI-compatible provider** вместо отдельного OpenRouterProvider:
   оба endpoint'а говорят на одном протоколе; rich-метаданные OpenRouter
   парсятся опционально с graceful degradation к OpenAI-формату.
3. **validate_model при смене модели** (llm_set), а не перед каждым запросом:
   per-request validation добавляла бы HTTP roundtrip к каждому ответу; intent
   (fail-fast на несуществующей модели) выполняется в момент переключения.
