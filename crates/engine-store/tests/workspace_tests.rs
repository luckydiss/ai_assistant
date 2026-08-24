mod common;

use engine_store::{ContextRow, MeetingRow, SessionStore};
use std::path::PathBuf;

fn open_store(name: &str) -> (SessionStore, PathBuf) {
    let dir: PathBuf = common::temp_dir(&format!("workspace-{name}"));
    let db = dir.join("test.db");
    (SessionStore::open(db.to_str().unwrap()).unwrap(), db)
}

#[test]
fn meeting_create_list() {
    let (store, _) = open_store("create-list");
    let a = store.create_meeting("интервью 1", "rust dev").unwrap();
    let b = store.create_meeting("интервью 2", "").unwrap();

    let list = store.list_meetings().unwrap();
    assert_eq!(list.len(), 2);
    let ids: Vec<String> = list.iter().map(|m| m.id.clone()).collect();
    assert!(ids.contains(&a));
    assert!(ids.contains(&b));
    let m = list.iter().find(|m| m.id == a).unwrap();
    assert_eq!(m.name, "интервью 1");
    assert_eq!(m.vacancy, "rust dev");
    assert!(m.context_id.is_none());
    assert_eq!(m.messages, 0);
}

#[test]
fn meeting_rename_delete() {
    let (store, _) = open_store("rename-delete");
    let id = store.create_meeting("старое", "").unwrap();
    store.rename_meeting(&id, "новое").unwrap();
    let list = store.list_meetings().unwrap();
    assert_eq!(list[0].name, "новое");

    store.delete_meeting(&id).unwrap();
    let list = store.list_meetings().unwrap();
    assert!(list.is_empty());
}

#[test]
fn meeting_counters_update() {
    let (store, _) = open_store("counters");
    let id = store.create_meeting("m", "").unwrap();
    store.bump_messages(&id, 2).unwrap();
    store.bump_messages(&id, 3).unwrap();
    let list = store.list_meetings().unwrap();
    assert_eq!(list[0].messages, 5);
}

#[test]
fn start_session_upsert() {
    let (store, db) = open_store("upsert");
    store.start_session("s1", "{}").unwrap();
    store.end_session("s1").unwrap();
    store.start_session("s1", "{}").unwrap();

    let conn = rusqlite::Connection::open(&db).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE id = ?1",
            rusqlite::params!["s1"],
            |r| r.get(0),
        )
        .unwrap();
    let ended: Option<String> = conn
        .query_row(
            "SELECT ended_at FROM sessions WHERE id = ?1",
            rusqlite::params!["s1"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    assert!(ended.is_none());
}

#[test]
fn context_roundtrip() {
    let (store, _) = open_store("context-roundtrip");
    let c = ContextRow {
        id: "ctx-1".into(),
        name: "Python Backend".into(),
        role: "Сеньор Python".into(),
        languages: vec!["ru".into(), "en".into()],
        resume_text: "5 лет Django".into(),
        extra_prompt: "Говори кратко".into(),
    };
    store.create_context(&c).unwrap();

    let got = store.get_context("ctx-1").unwrap();
    assert_eq!(got.name, c.name);
    assert_eq!(got.role, c.role);
    assert_eq!(got.languages, vec!["ru", "en"]);
    assert_eq!(got.resume_text, "5 лет Django");
    assert_eq!(got.extra_prompt, "Говори кратко");

    let mut updated = c;
    updated.name = "Python Backend v2".into();
    updated.languages = vec!["ru".into()];
    store.update_context(&updated).unwrap();
    let got = store.get_context("ctx-1").unwrap();
    assert_eq!(got.name, "Python Backend v2");
    assert_eq!(got.languages, vec!["ru"]);

    store.delete_context("ctx-1").unwrap();
    assert!(store.get_context("ctx-1").is_err());
    assert!(store.list_contexts().unwrap().is_empty());
}

#[test]
fn active_context_per_meeting() {
    let (store, _) = open_store("per-meeting");
    let c = ContextRow {
        id: "ctx-1".into(),
        name: "c".into(),
        role: "r".into(),
        languages: vec![],
        resume_text: String::new(),
        extra_prompt: String::new(),
    };
    store.create_context(&c).unwrap();
    let m = store.create_meeting("m", "").unwrap();
    store.set_meeting_context(&m, "ctx-1").unwrap();

    let list = store.list_meetings().unwrap();
    assert_eq!(list[0].context_id.as_deref(), Some("ctx-1"));
}

#[test]
fn import_resume_text() {
    let (store, _) = open_store("import-resume");
    let c = ContextRow {
        id: "ctx-1".into(),
        name: "c".into(),
        role: "r".into(),
        languages: vec!["ru".into()],
        resume_text: "Опыт 3 года в Rust".into(),
        extra_prompt: String::new(),
    };
    store.create_context(&c).unwrap();
    let got = store.get_context("ctx-1").unwrap();
    assert!(got.resume_text.contains("Rust"));
    assert_eq!(got.languages, vec!["ru"]);
}

#[test]
fn meeting_row_serde() {
    let m = MeetingRow {
        id: "a".into(),
        name: "n".into(),
        vacancy: "".into(),
        context_id: Some("c".into()),
        created_at: "t".into(),
        messages: 3,
    };
    let s = serde_json::to_string(&m).unwrap();
    let back: MeetingRow = serde_json::from_str(&s).unwrap();
    assert_eq!(back.id, "a");
    assert_eq!(back.messages, 3);
}
