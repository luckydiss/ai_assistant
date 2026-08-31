# Delta: Orchestrator (memory layers)

## ADDED Requirements

### Requirement: Key Turn Detection
is_key_turn(t) SHALL возвращать true, если реплика содержит вопрос/техн. маркер («напиши», «объясни», «расскажи», «как работает», «почему», «что будет», «код», «сложность») ИЛИ длиннее 200 символов.

#### Scenario: Вопрос детектится (test: key_question_detected)
- GIVEN I «Объясните, как работает event loop»
- THEN is_key_turn = true

#### Scenario: Короткая реплика не детектится (test: short_not_key)
- GIVEN C «да, понял»
- THEN is_key_turn = false

### Requirement: Key Turns Buffer
При on_turn, если is_key_turn, реплика SHALL добавляться в key_turns (cap key_turns_cap, FIFO).

#### Scenario: Кап ключевых (test: key_turns_cap)
- GIVEN cap=12 и 15 ключевых реплик
- THEN key_turns.len()==12, старейшие две выброшены

### Requirement: Recent Window Drain
После on_turn, если turns.len() > recent_window, старейшие (len - recent_window) SHALL извлекаться и отправляться в суммаризацию; turns остаются ≤ recent_window.

#### Scenario: Окно ограничено (test: recent_window_drain)
- GIVEN recent_window=12 и 15 turn
- WHEN обработаны
- THEN turns.len()==12, 3 ушли в суммаризацию

### Requirement: Periodic Summarization
Суммаризация SHALL асинхронно сжимать current_summary + drained в 2-4 предложения и обновлять summary активного чата (Cmd::SummaryDone), не блокируя on_turn.

#### Scenario: Summary обновляется (test: summary_updates)
- GIVEN mock complete возвращает «RES»
- WHEN drain сработал
- THEN summary активного чата == «RES»

## MODIFIED / REMOVED: (none)
