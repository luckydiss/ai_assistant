# Delta: Context (layered composition)

## MODIFIED Requirements

### Requirement: Message Assembly
ContextBuilder SHALL принимать ContextInput {summary, key_turns, recent, focus, note} и компоновать user-блок в порядке: Резюме сессии → Ключевые моменты → Недавние реплики → Последний вопрос I → Комментарий → «Ответь по запросу кандидата.».

#### Scenario: Все слои в промпте (test: builds_all_layers)
- GIVEN summary="S", key_turns=[k1], recent=[r1,r2], focus Some, note Some
- WHEN build()
- THEN user-блок содержит "S", k1, r1, r2, focus, note в указанном порядке

#### Scenario: Пустые слои пропускаются (test: skips_empty_layers)
- GIVEN summary="", key_turns=[]
- WHEN build()
- THEN user-блок не содержит строк "Резюме" и "Ключевые моменты"

#### Scenario: Бюджет-страховка (test: budget_safety)
- GIVEN recent из 200 длинных turns
- WHEN build() с max_tokens=8000
- THEN оценка токенов результата ≤ 8000 (усечение recent с конца старейших)

## ADDED Requirements

### Requirement: ContextInput Struct
Система SHALL предоставлять ContextInput<'a> с полями summary:&str, key_turns:&[Turn], recent:&[Turn], focus:Option<&Turn>, note:Option<&str>.

#### Scenario: Конструктор (test: context_input_fields)

## REMOVED Requirements
(прежняя сигнатура build(summary, turns, focus, note) заменяется; вызовы и тесты обновить)
