mod common;

use engine_store::SessionStore;
use std::path::PathBuf;

#[test]
fn creates_session_and_turns() {
    let dir: PathBuf = common::temp_dir("sqlite");
    let db = dir.join("test.db");
    let store = SessionStore::open(db.to_str().unwrap()).unwrap();
    store
        .start_session("s1", r#"{"llm":{"model":"deepseek-v4-flash-0731"}}"#)
        .unwrap();
    store
        .insert_turn(
            "s1",
            "Interviewer",
            "привет",
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:01Z",
        )
        .unwrap();
    store
        .insert_turn(
            "s1",
            "Candidate",
            "здравствуйте",
            "2026-01-01T00:00:01Z",
            "2026-01-01T00:00:02Z",
        )
        .unwrap();

    let conn = rusqlite::Connection::open(&db).unwrap();
    let turns: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM turns WHERE session_id = ?1",
            rusqlite::params!["s1"],
            |r| r.get(0),
        )
        .unwrap();
    let sessions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE id = ?1",
            rusqlite::params!["s1"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(turns, 2);
    assert_eq!(sessions, 1);

    store.end_session("s1").unwrap();
    let ended: Option<String> = conn
        .query_row(
            "SELECT ended_at FROM sessions WHERE id = ?1",
            rusqlite::params!["s1"],
            |r| r.get(0),
        )
        .unwrap();
    assert!(ended.is_some());
}

#[test]
fn records_answer_metrics() {
    let dir: PathBuf = common::temp_dir("sqlite-metrics");
    let db = dir.join("test.db");
    let store = SessionStore::open(db.to_str().unwrap()).unwrap();
    store.start_session("s1", "{}").unwrap();

    for (outcome, ttft) in [
        ("answered", 400),
        ("answered", 800),
        ("answered", 1200),
        ("error", 0),
        ("skipped", 0),
    ] {
        store
            .insert_answer("s1", "manual", outcome, "текст", 0, ttft)
            .unwrap();
    }

    let stats = store.stats("s1").unwrap();
    assert_eq!(stats.answered, 3);
    assert_eq!(stats.skipped, 1);
    assert_eq!(stats.errors, 1);
    assert_eq!(stats.p50_ttft_ms, 800);
    assert_eq!(stats.p95_ttft_ms, 1200);
}

#[test]
fn chats_roundtrip() {
    let dir: PathBuf = common::temp_dir("sqlite-chats");
    let db = dir.join("test.db");
    let store = SessionStore::open(db.to_str().unwrap()).unwrap();
    let meeting = store.create_meeting("собеседование", "Rust").unwrap();

    let c1 = store.create_chat(&meeting).unwrap();
    let c2 = store.create_chat(&meeting).unwrap();
    assert_eq!(c1.number, 1);
    assert_eq!(c2.number, 2);

    let chats = store.list_chats(&meeting).unwrap();
    assert_eq!(chats.len(), 2);
    assert_eq!(chats[0].number, 1);
    assert_eq!(chats[1].number, 2);

    store.set_chat_context(&c2.id, "ctx-1").unwrap();
    let chats = store.list_chats(&meeting).unwrap();
    assert_eq!(chats[1].context_id, "ctx-1");
}

#[test]
fn notes_roundtrip() {
    let dir: PathBuf = common::temp_dir("sqlite-notes");
    let db = dir.join("test.db");
    let store = SessionStore::open(db.to_str().unwrap()).unwrap();

    let n1 = store.create_note("Заметка 1", "текст").unwrap();
    let n2 = store.create_note("Заметка 2", "другой").unwrap();

    let all = store.notes_list().unwrap();
    assert_eq!(all.len(), 2);

    let got = store.note_get(&n1).unwrap();
    assert_eq!(got.name, "Заметка 1");
    assert_eq!(got.text, "текст");
    assert_ne!(n1, n2);
}

#[test]
fn chat_msgs_roundtrip() {
    let dir: PathBuf = common::temp_dir("sqlite-chatmsgs");
    let db = dir.join("test.db");
    let store = SessionStore::open(db.to_str().unwrap()).unwrap();
    let meeting = store.create_meeting("собеседование", "").unwrap();
    let chat = store.create_chat(&meeting).unwrap();

    let msgs = vec![
        engine_store::ChatMsg {
            speaker: "I".into(),
            text: "что такое ml".into(),
            at: "t1".into(),
        },
        engine_store::ChatMsg {
            speaker: "C".into(),
            text: "ML это ...".into(),
            at: "t2".into(),
        },
    ];
    store.save_chat_msgs(&chat.id, &msgs).unwrap();

    let loaded = store.chat_msgs(&chat.id).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].speaker, "I");
    assert_eq!(loaded[0].text, "что такое ml");
    assert_eq!(loaded[1].text, "ML это ...");
}
