use crate::StoreError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

pub struct SessionStore {
    conn: Connection,
}

pub struct LatencyStats {
    pub p50_ttft_ms: u64,
    pub p95_ttft_ms: u64,
    pub answered: u64,
    pub skipped: u64,
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRow {
    pub id: String,
    pub name: String,
    pub vacancy: String,
    pub context_id: Option<String>,
    pub created_at: String,
    pub messages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRow {
    pub id: String,
    pub name: String,
    pub role: String,
    pub languages: Vec<String>,
    pub resume_text: String,
    pub extra_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRow {
    pub id: String,
    pub meeting_id: String,
    pub number: i64,
    pub context_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRow {
    pub id: String,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    pub speaker: String,
    pub text: String,
    pub at: String,
}

impl SessionStore {
    pub fn open(path: &str) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY, started_at TEXT NOT NULL,
                ended_at TEXT, config_json TEXT);
             CREATE TABLE IF NOT EXISTS turns (
                id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL,
                speaker TEXT NOT NULL, text TEXT NOT NULL,
                start_time TEXT NOT NULL, end_time TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS answers (
                id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT NOT NULL,
                trigger_kind TEXT NOT NULL, outcome TEXT NOT NULL,
                full_text TEXT, stt_latency_ms INTEGER, ttft_ms INTEGER,
                created_at TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS meetings (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, vacancy TEXT DEFAULT '',
                context_id TEXT, created_at TEXT NOT NULL, messages INTEGER DEFAULT 0);
             CREATE TABLE IF NOT EXISTS contexts (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, role TEXT DEFAULT '',
                languages TEXT DEFAULT '[]', resume_text TEXT DEFAULT '',
                extra_prompt TEXT DEFAULT '');
             CREATE TABLE IF NOT EXISTS chats (
                id TEXT PRIMARY KEY, meeting_id TEXT NOT NULL,
                number INTEGER NOT NULL, context_id TEXT DEFAULT '');
             CREATE TABLE IF NOT EXISTS chat_msgs (
                id INTEGER PRIMARY KEY AUTOINCREMENT, chat_id TEXT NOT NULL,
                speaker TEXT NOT NULL, text TEXT NOT NULL, at TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, text TEXT DEFAULT '');",
        )?;
        Ok(Self { conn })
    }

    pub fn start_session(&self, id: &str, config_json: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO sessions (id, started_at, config_json) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET ended_at=NULL, started_at=?2",
            params![id, chrono::Utc::now().to_rfc3339(), config_json],
        )?;
        Ok(())
    }

    pub fn end_session(&self, id: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
            params![chrono::Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    // --- meetings ---

    pub fn create_meeting(&self, name: &str, vacancy: &str) -> Result<String, StoreError> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO meetings (id, name, vacancy, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, vacancy, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(id)
    }

    pub fn list_meetings(&self) -> Result<Vec<MeetingRow>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, vacancy, context_id, created_at, messages FROM meetings ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |r| {
            Ok(MeetingRow {
                id: r.get(0)?,
                name: r.get(1)?,
                vacancy: r.get(2)?,
                context_id: r.get(3)?,
                created_at: r.get(4)?,
                messages: r.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn rename_meeting(&self, id: &str, name: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE meetings SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        Ok(())
    }

    pub fn delete_meeting(&self, id: &str) -> Result<(), StoreError> {
        self.conn.execute("DELETE FROM meetings WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn bump_messages(&self, meeting_id: &str, n: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE meetings SET messages = messages + ?1 WHERE id = ?2",
            params![n, meeting_id],
        )?;
        Ok(())
    }

    pub fn set_meeting_context(&self, meeting_id: &str, context_id: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE meetings SET context_id = ?1 WHERE id = ?2",
            params![context_id, meeting_id],
        )?;
        Ok(())
    }

    // --- contexts ---

    pub fn create_context(&self, c: &ContextRow) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO contexts (id, name, role, languages, resume_text, extra_prompt) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                c.id,
                c.name,
                c.role,
                serde_json::to_string(&c.languages)?,
                c.resume_text,
                c.extra_prompt
            ],
        )?;
        Ok(())
    }

    pub fn get_context(&self, id: &str) -> Result<ContextRow, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, role, languages, resume_text, extra_prompt FROM contexts WHERE id = ?1",
        )?;
        let row = stmt.query_row(params![id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;
        Ok(ContextRow {
            id: row.0,
            name: row.1,
            role: row.2,
            languages: serde_json::from_str(&row.3)?,
            resume_text: row.4,
            extra_prompt: row.5,
        })
    }

    pub fn update_context(&self, c: &ContextRow) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE contexts SET name=?1, role=?2, languages=?3, resume_text=?4, extra_prompt=?5 WHERE id=?6",
            params![
                c.name,
                c.role,
                serde_json::to_string(&c.languages)?,
                c.resume_text,
                c.extra_prompt,
                c.id
            ],
        )?;
        Ok(())
    }

    pub fn delete_context(&self, id: &str) -> Result<(), StoreError> {
        self.conn.execute("DELETE FROM contexts WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_contexts(&self) -> Result<Vec<ContextRow>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, role, languages, resume_text, extra_prompt FROM contexts ORDER BY name",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, name, role, langs, resume_text, extra_prompt) = row?;
            out.push(ContextRow {
                id,
                name,
                role,
                languages: serde_json::from_str(&langs)?,
                resume_text,
                extra_prompt,
            });
        }
        Ok(out)
    }

    pub fn insert_turn(
        &self,
        session_id: &str,
        speaker: &str,
        text: &str,
        start: &str,
        end: &str,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO turns (session_id, speaker, text, start_time, end_time) VALUES (?1,?2,?3,?4,?5)",
            params![session_id, speaker, text, start, end],
        )?;
        Ok(())
    }

    pub fn insert_answer(
        &self,
        session_id: &str,
        trigger_kind: &str,
        outcome: &str,
        full_text: &str,
        stt_latency_ms: u64,
        ttft_ms: u64,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO answers (session_id, trigger_kind, outcome, full_text, stt_latency_ms, ttft_ms, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![session_id, trigger_kind, outcome, full_text, stt_latency_ms as i64, ttft_ms as i64, chrono::Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn stats(&self, session_id: &str) -> Result<LatencyStats, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT outcome, COUNT(*) FROM answers WHERE session_id = ?1 GROUP BY outcome",
        )?;
        let mut answered = 0u64;
        let mut skipped = 0u64;
        let mut errors = 0u64;
        let rows = stmt.query_map(params![session_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (outcome, cnt) = row?;
            match outcome.as_str() {
                "answered" => answered = cnt as u64,
                "skipped" => skipped = cnt as u64,
                _ => errors = cnt as u64,
            }
        }

        let ttfts: Vec<i64> = {
            let mut stmt = self.conn.prepare(
                "SELECT ttft_ms FROM answers WHERE session_id = ?1 AND outcome = 'answered' ORDER BY ttft_ms",
            )?;
            let rows = stmt.query_map(params![session_id], |r| r.get::<_, i64>(0))?;
            let mut v = Vec::new();
            for row in rows {
                v.push(row?);
            }
            v
        };
        let q = |p: f64| -> u64 {
            if ttfts.is_empty() {
                return 0;
            }
            let idx = ((ttfts.len() as f64 - 1.0) * p).round() as usize;
            ttfts[idx] as u64
        };

        Ok(LatencyStats {
            p50_ttft_ms: q(0.5),
            p95_ttft_ms: q(0.95),
            answered,
            skipped,
            errors,
        })
    }

    // --- chats ---

    pub fn create_chat(&self, meeting_id: &str) -> Result<ChatRow, StoreError> {
        let id = uuid::Uuid::new_v4().to_string();
        let number: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(number), 0) + 1 FROM chats WHERE meeting_id = ?1",
            params![meeting_id],
            |r| r.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO chats (id, meeting_id, number) VALUES (?1, ?2, ?3)",
            params![id, meeting_id, number],
        )?;
        Ok(ChatRow {
            id,
            meeting_id: meeting_id.to_string(),
            number,
            context_id: String::new(),
        })
    }

    pub fn list_chats(&self, meeting_id: &str) -> Result<Vec<ChatRow>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, meeting_id, number, context_id FROM chats WHERE meeting_id = ?1 ORDER BY number",
        )?;
        let rows = stmt.query_map(params![meeting_id], |r| {
            Ok(ChatRow {
                id: r.get(0)?,
                meeting_id: r.get(1)?,
                number: r.get(2)?,
                context_id: r.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn set_chat_context(&self, chat_id: &str, context_id: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE chats SET context_id = ?1 WHERE id = ?2",
            params![context_id, chat_id],
        )?;
        Ok(())
    }

    // --- chat messages ---

    pub fn save_chat_msgs(&self, chat_id: &str, msgs: &[ChatMsg]) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM chat_msgs WHERE chat_id = ?1",
            params![chat_id],
        )?;
        for m in msgs {
            self.conn.execute(
                "INSERT INTO chat_msgs (chat_id, speaker, text, at) VALUES (?1, ?2, ?3, ?4)",
                params![chat_id, m.speaker, m.text, m.at],
            )?;
        }
        Ok(())
    }

    pub fn chat_msgs(&self, chat_id: &str) -> Result<Vec<ChatMsg>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT speaker, text, at FROM chat_msgs WHERE chat_id = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![chat_id], |r| {
            Ok(ChatMsg {
                speaker: r.get(0)?,
                text: r.get(1)?,
                at: r.get(2)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    // --- notes ---

    pub fn create_note(&self, name: &str, text: &str) -> Result<String, StoreError> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO notes (id, name, text) VALUES (?1, ?2, ?3)",
            params![id, name, text],
        )?;
        Ok(id)
    }

    pub fn notes_list(&self) -> Result<Vec<NoteRow>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, text FROM notes ORDER BY name")?;
        let rows = stmt.query_map([], |r| {
            Ok(NoteRow {
                id: r.get(0)?,
                name: r.get(1)?,
                text: r.get(2)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn note_get(&self, id: &str) -> Result<NoteRow, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, text FROM notes WHERE id = ?1")?;
        let row = stmt.query_row(params![id], |r| {
            Ok(NoteRow {
                id: r.get(0)?,
                name: r.get(1)?,
                text: r.get(2)?,
            })
        })?;
        Ok(row)
    }
}
