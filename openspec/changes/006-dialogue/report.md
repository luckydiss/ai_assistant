# Report: Dialogue Assembler

## Result

`crates/engine-dialogue` реализован: `Assembler` с reorder (BinaryHeap + сортированная вставка), merge коротких пауз, dedup, garbage-filter и rolling summary. 11/11 тестов, clippy/fmt/release/workspace чисты, example работает.

## Deviations from Design

1. **`Transcript` получил `Ord`/`PartialOrd`:** design требует `BinaryHeap<Reverse<Transcript>>`, но `Transcript` в design.md §3 без трейтов сравнения. Реализовано сравнение по `(start_time, speaker.lane, text)` — это же даёт stable order для `handles_same_timestamp` (Interviewer lane 0 < Candidate lane 1). `Speaker::lane` — `pub(crate)`.

2. **Reorder через `insertion_index`, не `push`:** design §4 вставляет новые turns `self.turns.push(...)` — это ломает out-of-order сценарий (`orders_by_timestamp`): T2 раньше T1, но пришёл позже. Реализована вставка по позиции `partition_point((start_time, lane))`, поэтому `turns` всегда хронологически упорядочены.

3. **Merge/dedup против *предшественника*, не `last()`:** из-за (2) «последний» turn не всегда предшествующий по времени. `is_duplicate`/`can_merge` сравнивают с `turns[idx - 1]` (тот, кто реально перед транскриптом по таймлайну).

4. **`can_merge` — строго `< 500мс`:** spec: «пауза **<** 500мс». `<=` ломал `keeps_similar_text` (пауза ровно 500мс склеивала «Hello world» + «Hello world!», а спека требует оставить оба). Диапазон `0..merge_threshold_ms`.

5. **`is_garbage` — только filler-лист, без «<2 слов»:** правило дизайна «<2 words → garbage» конфликтовало бы с:
   - `filters_exact_duplicate` (1 слово «Hello» должно остаться, чтобы проверить dedup),
   - `keeps_valid_short_reply` (1 слово «Да» должно остаться).
   Реализовано: garbage только если текст входит в список fillers (`ок/okay/хорошо/ага/угу/спасибо`). «да/нет» намеренно **не** в списке (валидные короткие ответы). «ок» из `filters_short_utterance` фильтруется как filler.

6. **Без `unwrap`:** дизайн использовал `self.turns.last().unwrap()` в `is_duplicate`. Заменено на проверку `idx == 0 → false` (no-unwrap политика project.md).

7. **`impl Default for Assembler`:** clippy `new_without_default` в `#![deny(clippy::all)]`.

8. **`async` в `process_transcript`/`process_buffer`/`generate_summary` сохранён** (по дизайну), хотя сейчас await не требуется — задел под LLM-суммаризацию в 008-llm.

9. **Unused variants `ChannelClosed`, `SummaryFailed`** в `DialogueError` сохранены по дизайну (публичные, не dead-code).

## Verified

- `cargo test -p engine-dialogue` — 11/11 ok
- `cargo clippy -p engine-dialogue --all-targets -- -D warnings` — ok
- `cargo build -p engine-dialogue --release` — ok
- `cargo build --workspace` — ok
- `cargo fmt -p engine-dialogue --check` — ok
- `cargo run -p engine-dialogue --example dialogue_demo` (manual, 7.2) — T2 (Candidate, раньше) выводится первым, T1 вторым: reorder работает; garbage-фильтр не срабатывает на демо-текстах (2+ слова). Запуск требует `RUST_LOG=info` — `fmt().init()` читает EnvFilter из env.