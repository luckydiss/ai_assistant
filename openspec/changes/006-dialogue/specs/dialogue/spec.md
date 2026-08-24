# Delta: Dialogue Assembler

## ADDED Requirements

### Requirement: Timeline Ordering
Система SHALL упорядочивать транскрипты по timestamp начала, а не по времени получения.

#### Scenario: Out-of-order transcripts (test: orders_by_timestamp)
- GIVEN два транскрипта: T1 (start=1000ms) и T2 (start=500ms)
- WHEN T2 приходит после T1 из-за задержки STT
- THEN в диалоге T2 идет первым
- AND диалог корректно упорядочен по времени

#### Scenario: Same timestamp handling (test: handles_same_timestamp)
- GIVEN два транскрипта с одинаковым start time
- WHEN оба обработаны
- THEN порядок определяется по lane (I before C)
- AND нет конфликтов

### Requirement: Phrase Merging
Система SHALL склеивать обрывки одной фразы если пауза между ними < 500мс.

#### Scenario: Short pause merge (test: merges_short_pause)
- GIVEN две реплики от одного speaker с паузой 200мс
- WHEN обработаны assembler
- THEN объединены в одну реплику
- AND текст склеен с пробелом

#### Scenario: Long pause split (test: splits_long_pause)
- GIVEN две реплики от одного speaker с паузой 800мс
- WHEN обработаны assembler
- THEN остаются как две отдельные реплики
- AND не объединяются

### Requirement: Deduplication
Система SHALL фильтровать повторяющиеся транскрипты (галлюцинации Whisper).

#### Scenario: Exact duplicate (test: filters_exact_duplicate)
- GIVEN транскрипт "Hello" и повторный "Hello" от того же speaker
- WHEN второй приходит в течение 2 секунд
- THEN второй отфильтрован
- AND в диалоге только один "Hello"

#### Scenario: Similar text (test: keeps_similar_text)
- GIVEN "Hello world" и "Hello world!"
- WHEN оба обработаны
- THEN оба остаются в диалоге
- AND не считаются дубликатами

### Requirement: Garbage Filtering
Система SHALL фильтровать короткие и бессмысленные реплики.

#### Scenario: Short utterance (test: filters_short_utterance)
- GIVEN реплика с 1 словом "ок"
- WHEN обработана
- THEN отфильтрована
- AND не добавлена в диалог

#### Scenario: Filler word (test: filters_filler_word)
- GIVEN реплика "ага" или "хорошо"
- WHEN обработана
- THEN отфильтрована
- AND не добавлена в диалог

#### Scenario: Valid short reply (test: keeps_valid_short_reply)
- GIVEN реплика "Да" как ответ на вопрос
- WHEN обработана
- THEN добавлена в диалог
- AND не отфильтрована

### Requirement: Rolling Summary
Система SHALL сжимать старую часть диалога в summary каждые 16 turns.

#### Scenario: Summary generation (test: generates_summary)
- GIVEN диалог с 20 turns
- WHEN достигнут threshold
- THEN первые 4 turns сжаты в 2-3 предложения summary
- AND summary сохраняется отдельно
- AND оригинальные turns удалены из основного диалога

#### Scenario: Summary update (test: updates_summary)
- GIVEN существующий summary и новые 16 turns
- WHEN trigger срабатывает
- THEN новый summary включает старый + новые turns
- AND длина summary остается ≤ 3 предложения

## MODIFIED Requirements

(none)

## REMOVED Requirements

(none)
