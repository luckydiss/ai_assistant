# Delta: Store (chats, notes)

## ADDED Requirements

### Requirement: Chats Table
Система SHALL хранить чаты: id, meeting_id, number, context_id; методы create_chat, list_chats(meeting), set_chat_context.

#### Scenario: Roundtrip (test: chats_roundtrip)

### Requirement: Notes Table
Система SHALL хранить заметки: id, name, text; методы notes_list (CRUD-UI — в 024).

#### Scenario: Roundtrip (test: notes_roundtrip)

## MODIFIED / REMOVED: (none)
