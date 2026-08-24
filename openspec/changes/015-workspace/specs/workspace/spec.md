# Delta: Workspace

## ADDED Requirements

### Requirement: Meetings CRUD
Система SHALL поддерживать создание, список, переименование, удаление встреч.

#### Scenario: Создание и список (test: meeting_create_list)
- GIVEN create_meeting("Ozon", "вакансия...")
- WHEN list_meetings()
- THEN встреча присутствует с name, vacancy, created_at, messages=0

#### Scenario: Переименование и удаление (test: meeting_rename_delete)
- GIVEN встреча существует
- WHEN rename затем delete
- THEN list_meetings() не содержит её

### Requirement: Meeting Metrics
Система SHALL инкрементировать счётчик сообщений встречи при каждом turn и answer.

#### Scenario: Счётчик растёт (test: meeting_counters_update)
- GIVEN встреча и 2 insert_turn + 1 insert_answer
- WHEN list_meetings()
- THEN messages = 3

### Requirement: Resume Meeting
Система SHALL позволять продолжить встречу: сессия дописывается в ту же запись.

#### Scenario: Resume не дублирует (test: resume_appends)
- GIVEN встреча с завершённой сессией
- WHEN start_session(тот же id) повторно
- THEN строка meetings одна, turns дописываются

### Requirement: Contexts CRUD
Система SHALL поддерживать контексты с полями name, role, languages, resume_text, extra_prompt.

#### Scenario: Roundtrip (test: context_roundtrip)
- GIVEN create_context со всеми полями
- WHEN get_context
- THEN все поля совпадают

#### Scenario: Активный контекст встречи (test: active_context_per_meeting)
- GIVEN встреча и два контекста
- WHEN set_meeting_context(meeting, ctx2)
- THEN active_context(meeting) = ctx2

### Requirement: Resume Import
Система SHALL принимать текст резюме из TXT/MD, переданный с фронтенда.

#### Scenario: Импорт текста (test: import_resume_text)
- GIVEN update_context(resume_text = "5 лет Rust...")
- WHEN get_context
- THEN resume_text сохранён

## MODIFIED Requirements
(none в workspace)

## REMOVED Requirements
(none)
