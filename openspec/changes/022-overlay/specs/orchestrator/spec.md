# Delta: Orchestrator (multi-chat)

## MODIFIED Requirements

### Requirement: Context-Only Turns
Оркестратор SHALL вести истории раздельно по чатам: turns и summary хранятся per chat_id; on_turn пишет в активный чат; manual использует историю активного чата.

#### Scenario: Изоляция чатов (test: chats_isolated)
- GIVEN чат 1 с 2 turn, переключение на чат 2
- WHEN manual(None) в чате 2
- THEN тело запроса НЕ содержит реплики чата 1

#### Scenario: Активный чат получает turn (test: active_chat_gets_turns)
- GIVEN активен чат 2
- WHEN on_turn
- THEN реплика в истории чата 2

## ADDED Requirements

### Requirement: Chat Lifecycle
Система SHALL поддерживать set_active_chat(id), reset_active() (очистка истории+summary активного чата).

#### Scenario: Reset (test: reset_active_clears)
- GIVEN чат с 3 turn
- WHEN reset_active()
- THEN следующий manual шлёт пустую историю

## REMOVED: (none)
