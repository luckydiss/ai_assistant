# Delta: UI (model modal rewrite)

## MODIFIED Requirements

### Requirement: Model Selection Modal
Модалка SHALL показывать модели с метаданными: имя, семейство, context_length, pricing. Группировка по family из метаданных (models_list возвращает объекты), без regex в UI.

#### Scenario: Группировка по метаданным (test: modal_groups_by_family)
- WHEN открыта модалка
- THEN семейства из m.family моделей_list, не из regex

#### Scenario: Фильтрация чатовых (test: modal_filters_chat)
- WHEN список моделей
- THEN только chat=true модели (фильтрует бэкенд)

### Requirement: Reasoning Effort Slider
Слайдер SHALL отправлять reasoning_effort только если выбран (не null).

#### Scenario: Выкл не отправляет (test: effort_off_not_sent)
- GIVEN слайдер в позиции 0
- WHEN сохранение
- THEN reasoning_effort не передаётся в запрос

## ADDED Requirements

### Requirement: Model Metadata Display
Модалка SHALL показывать для каждой модели: context_length, pricing (input/output), capabilities badges.

#### Scenario: Метаданные видны (test: manual_metadata_display)
- WHEN открыта модалка
- THEN видны context_length, pricing, capabilities (manual)

## REMOVED Requirements
(Стоп-слова, familyOf regex, хардкод цветов — заменяются метаданными)
