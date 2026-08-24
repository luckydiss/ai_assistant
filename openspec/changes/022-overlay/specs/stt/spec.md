# Delta: STT (language)

## MODIFIED Requirements

### Requirement: Groq API Client
Запрос SHALL содержать параметр language, если [stt] language != "auto" (иначе авто-детект).

#### Scenario: language в теле (test: stt_language_sent)
- GIVEN language="ru"
- WHEN транскрипция на mock с захватом тела
- THEN multipart-тело содержит поле language="ru"

## ADDED / REMOVED: (none)
