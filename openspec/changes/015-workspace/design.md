# Design: Workspace

## 1. Схема sqlite (добавить в SessionStore::open)

```sql
CREATE TABLE IF NOT EXISTS meetings (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, vacancy TEXT DEFAULT '',
  context_id TEXT, created_at TEXT NOT NULL, messages INTEGER DEFAULT 0);
CREATE TABLE IF NOT EXISTS contexts (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, role TEXT DEFAULT '',
  languages TEXT DEFAULT '[]', resume_text TEXT DEFAULT '',
  extra_prompt TEXT DEFAULT '');
```

## 2. Новые методы SessionStore (сигнатуры)

```rust
// meetings
pub fn create_meeting(&self, name: &str, vacancy: &str) -> Result<String, StoreError> // returns uuid
pub fn list_meetings(&self) -> Result<Vec<MeetingRow>, StoreError> // id,name,vacancy,context_id,created_at,messages + SUM(duration) из sessions по желанию
pub fn rename_meeting(&self, id: &str, name: &str) -> Result<(), StoreError>
pub fn delete_meeting(&self, id: &str) -> Result<(), StoreError>
pub fn bump_messages(&self, meeting_id: &str, n: i64) -> Result<(), StoreError>

// contexts
pub fn create_context(&self, c: &ContextRow) -> Result<(), StoreError>
pub fn get_context(&self, id: &str) -> Result<ContextRow, StoreError>
pub fn update_context(&self, c: &ContextRow) -> Result<(), StoreError>
pub fn delete_context(&self, id: &str) -> Result<(), StoreError>
pub fn list_contexts(&self) -> Result<Vec<ContextRow>, StoreError>
pub fn set_meeting_context(&self, meeting_id: &str, context_id: &str) -> Result<(), StoreError>

#[derive(Serialize, Deserialize, Clone)]
pub struct MeetingRow { pub id: String, pub name: String, pub vacancy: String,
    pub context_id: Option<String>, pub created_at: String, pub messages: i64 }
#[derive(Serialize, Deserialize, Clone)]
pub struct ContextRow { pub id: String, pub name: String, pub role: String,
    pub languages: Vec<String>, pub resume_text: String, pub extra_prompt: String }
```

`start_session` изменить на `INSERT INTO sessions ... ON CONFLICT(id) DO UPDATE SET ended_at=NULL, started_at=?1`.

## 3. Интеграция ContextBuilder

Новая структура входа вместо (system, persona):

```rust
pub struct PromptContext {
    pub base_system: String,   // из config.prompts.system
    pub role: String,
    pub extra_prompt: String,
    pub resume_text: String,
    pub vacancy: String,
}

impl ContextBuilder {
    pub fn with_workspace(base_system: String, ws: &PromptContext, max_tokens: usize) -> Self {
        let mut system = base_system;
        if !ws.extra_prompt.is_empty() { system.push_str(&format!("\n\n{}", ws.extra_prompt)); }
        let mut persona = ws.role.clone();
        if !ws.resume_text.is_empty() { persona.push_str(&format!("\nРезюме кандидата: {}", ws.resume_text)); }
        if !ws.vacancy.is_empty() { persona.push_str(&format!("\nВакансия: {}", ws.vacancy)); }
        Self { system, persona, max_tokens }
    }
}
```

Старый `new()` оставить как делегат с пустыми полями (обратная совместимость с тестами 007).

## 4. IPC-команды (apps/desktop/src/commands.rs — новый файл)

```rust
#[tauri::command] pub async fn meetings_list(state: State<'_, Arc<SessionStore>>) -> Vec<MeetingRow>
#[tauri::command] pub async fn meeting_create(state, name: String, vacancy: String) -> String
#[tauri::command] pub async fn meeting_rename(state, id: String, name: String)
#[tauri::command] pub async fn meeting_delete(state, id: String)
#[tauri::command] pub async fn contexts_list(state) -> Vec<ContextRow>
#[tauri::command] pub async fn context_save(state, ctx: ContextRow)
#[tauri::command] pub async fn context_delete(state, id: String)
#[tauri::command] pub async fn meeting_set_context(state, meeting_id: String, context_id: String)
```

Все команды — тонкие обёртки над store, логика в store (тестируется без Tauri).

## Рассмотрено и отклонено
- **Отдельная таблица languages:** отклонено — JSON-колонка проще
- **PDF-парсинг (pdf-extract):** отклонено для MVP
