# Delta: Model Modal Component (MODIFIED)

## MODIFIED Requirements

### Requirement: Model Selection Modal
Modal SHALL показывать модели с метаданными, группировать по семействам, поддерживать виртуализацию для 400+ моделей.

#### Scenario: Открытие модалки (test: modal_opens_and_loads)
- GIVEN пользователь кликнул на пилюлю модели в топбаре
- WHEN `openModelModal()` вызван
- THEN модалка появляется с fade-in анимацией (<200ms)
- AND загружаются модели через `commands.modelsList()`
- AND показывается skeleton loader пока данные загружаются

#### Scenario: Виртуализация списка (test: modal_virtualizes_400_models)
- GIVEN каталог содержит 417 моделей
- WHEN модалка открыта, выбрано семейство "Anthropic" (30 моделей)
- THEN в DOM рендерятся только видимые ~15 items (viewport height / item height)
- AND scroll плавный 60fps
- AND memory usage ≤ 15MB (vs 45MB без виртуализации)

#### Scenario: Группировка по семействам (test: modal_groups_by_family)
- GIVEN модели содержат семейства: Anthropic, OpenAI, Google, DeepSeek
- WHEN модалка рендерится
- THEN левая sidebar показывает 4 кнопки семейств (alphabetically sorted)
- AND клик на "Anthropic" фильтрует правый список только моделями Anthropic
- AND текущее семейство выделено accent color

#### Scenario: Метаданные модели (test: modal_shows_metadata)
- GIVEN модель `anthropic/claude-sonnet-5` с context_length=200000, pricing={input: 3.0, output: 15.0}
- WHEN карточка модели рендерится
- THEN показывает:
  - Имя: "Claude Sonnet 5"
  - ID: "anthropic/claude-sonnet-5" (monospace font)
  - Badges: "200k", "$3.00/$15.00", иконки vision/tools/reasoning (если есть)

#### Scenario: Выбор модели с валидацией (test: modal_selects_and_validates)
- GIVEN пользователь кликнул на модель "anthropic/claude-sonnet-5"
- WHEN `selectModel()` вызван
- THEN вызывается `commands.llmSet(id, currentEffort)`
- AND если валидация успешна → модалка закрывается, пилюля обновляется
- AND если валидация fail → toast error, модалка остаётся открытой

### Requirement: Reasoning Effort Slider (OLD: static labels; NEW: live preview)
Slider SHALL показывать 5 уровней с live preview токенов.

#### Scenario: Изменение effort (test: slider_changes_effort)
- GIVEN slider на позиции "Средний" (index 2 = "low")
- WHEN пользователь перемещает на "Высокий" (index 3 = "medium")
- THEN label обновляется: "Высокий"
- AND если модель поддерживает reasoning → показывается estimated overhead (~300 tokens)

#### Scenario: Применение effort (test: slider_applies_effort)
- GIVEN slider изменён на "Максимальный" (index 4 = "high")
- WHEN пользователь выбирает модель ИЛИ slider.onchange fired
- THEN вызывается `llmSet(currentModel, "high")`

### Requirement: Поиск по моделям (ADDED)
Modal SHALL фильтровать модели по имени/id/capabilities.

#### Scenario: Текстовый поиск (test: modal_search_text)
- GIVEN input содержит "sonnet"
- WHEN список фильтруется
- THEN показываются только модели с "sonnet" в name ИЛИ id (case-insensitive)
- AND counter показывает "12 models" (из 417 total)

#### Scenario: Фильтр по capabilities (test: modal_filter_capabilities)
- GIVEN включены фильтры: vision=true, tools=false
- WHEN список фильтруется
- THEN показываются только модели с `capabilities.vision === true`
- AND models без vision скрыты

#### Scenario: Комбинированный фильтр (test: modal_combined_filter)
- GIVEN search="claude" AND vision=true AND family="Anthropic"
- WHEN список фильтруется
- THEN показываются модели: (name/id contains "claude") AND (vision=true) AND (family="Anthropic")
- AND результат: 3 модели (например, claude-opus-4.8, claude-sonnet-4.6-vision, ...)

### Requirement: Keyboard Navigation (ADDED)
Modal SHALL поддерживать полную keyboard navigation.

#### Scenario: Tab navigation (test: modal_keyboard_tab)
- GIVEN модалка открыта
- WHEN пользователь нажимает Tab
- THEN фокус циклится: search input → family buttons → model list → effort slider → close button → search

#### Scenario: Arrow keys в списке (test: modal_keyboard_arrows)
- GIVEN фокус на модели "claude-sonnet-5" в списке
- WHEN пользователь нажимает ArrowDown
- THEN фокус переходит на следующую модель
- AND при достижении конца списка → wrap to first

#### Scenario: Enter выбирает модель (test: modal_keyboard_enter)
- GIVEN фокус на модели "claude-sonnet-5"
- WHEN пользователь нажимает Enter
- THEN модель выбирается (эквивалентно клику)

#### Scenario: Escape закрывает (test: modal_keyboard_escape)
- GIVEN модалка открыта
- WHEN пользователь нажимает Escape
- THEN модалка закрывается, фокус возвращается на пилюлю

## ADDED Requirements
(см. Поиск, Keyboard Navigation выше)

## REMOVED Requirements

### REMOVED: Host-first sidebar item
Старый UI показывал "api.dslab.tech" первым пунктом — удалено.

### REMOVED: Hardcoded family colors
Старый UI имел JS map `{OpenAI: "#10a37f", ...}` — заменён на CSS classes `.dot-f0..f7` (циклическая палитра).

### REMOVED: prettyName() function
Старый UI форматировал id клиентом ("gpt-4" → "GPT 4") — теперь бэкенд отдаёт human-readable `name`.
