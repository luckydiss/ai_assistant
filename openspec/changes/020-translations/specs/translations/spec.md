# Delta: Translations

## ADDED Requirements

### Requirement: Translation Pass
Система SHALL после Done переводить полный ответ на дополнительные языки контекста (skip(1).take(2)).

#### Scenario: Два доп. языка (test: translations_for_extra_langs)
- GIVEN languages = ["ru","en","de"] и mock-сервер со счётчиком
- WHEN ответ завершён (Done)
- THEN сервер получил ровно 2 запроса перевода после запроса ответа

#### Scenario: Больше трёх языков (test: max_two_extra_langs)
- GIVEN languages из 4 элементов
- WHEN Done
- THEN переводов ровно 2

#### Scenario: Один язык (test: single_lang_no_translation)
- GIVEN languages = ["ru"]
- WHEN Done
- THEN запросов перевода 0

### Requirement: Translation Quality Contract
Запрос перевода SHALL требовать сохранить markdown/код и не переводить идентификаторы.

#### Scenario: Инструкция в теле (test: translate_body_contains_lang)
- GIVEN translate(text, "en")
- WHEN mock захватил тело
- THEN тело содержит "en" и "markdown"

### Requirement: Translation Cancellation
Новый триггер SHALL отменять незавершённые переводы предыдущего ответа.

#### Scenario: Отмена (test: translation_cancel_on_new_fire)
- GIVEN медленный mock и первый Done уже запустил перевод
- WHEN приходит второй триггер
- THEN событие Translation первого ответа не приходит

### Requirement: UI Translation Blocks
Оверлей SHALL рендерить переводы под ответом с пометкой языка.

#### Scenario: Блоки видны (test: manual_translation_blocks)
- GIVEN контекст с 2 языками
- WHEN ответ готов
- THEN под ответом блок перевода (manual)

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
