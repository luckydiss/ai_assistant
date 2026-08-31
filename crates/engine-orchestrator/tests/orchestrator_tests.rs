mod mock;

use chrono::Utc;
use engine_context::ContextBuilder;
use engine_dialogue::{Speaker, Turn};
use engine_llm::LlmClient;
use engine_orchestrator::{gate, OrchEvent, Orchestrator};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

fn turn(speaker: Speaker, text: &str) -> Turn {
    let now = Utc::now();
    Turn {
        speaker,
        text: text.into(),
        start_time: now,
        end_time: now,
    
            typed: false,
    }
}

async fn orch(body: String, delay_ms: u64) -> (Orchestrator, Arc<std::sync::atomic::AtomicUsize>) {
    let (url, count) = mock::spawn_mock_sse(body, delay_ms).await;
    let llm = LlmClient::new(url, "k".into(), "m".into(), 0.0, 100, None).unwrap();
    let ctx = ContextBuilder::new("SYS".into(), "persona".into(), 4000);
    (Orchestrator::new(ctx, llm, false), count)
}

async fn orch_capture(
    body: String,
    body_out: Arc<Mutex<String>>,
) -> (Orchestrator, Arc<std::sync::atomic::AtomicUsize>) {
    let (url, count) = mock::spawn_mock_sse_capture(body, body_out).await;
    let llm = LlmClient::new(url, "k".into(), "m".into(), 0.0, 100, None).unwrap();
    let ctx = ContextBuilder::new("SYS".into(), "persona".into(), 4000);
    (Orchestrator::new(ctx, llm, false), count)
}

#[tokio::test]
async fn manual_trigger_fires() {
    let (orch, count) = orch(mock::sse_body(&["Привет"]), 0).await;

    let mut rx = orch.subscribe();
    orch.manual(Some("помоги с кодом".into()), None);

    let result = timeout(Duration::from_secs(5), async {
        let mut done = false;
        while let Ok(ev) = rx.recv().await {
            match ev {
                OrchEvent::Done { .. } => {
                    done = true;
                    break;
                }
                OrchEvent::Error { message, .. } => panic!("error: {message}"),
                _ => {}
            }
        }
        done
    })
    .await
    .expect("timed out");

    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert!(result);
}

#[tokio::test]
async fn manual_cancels_previous() {
    let (orch, count) = orch(mock::sse_body(&["Привет"]), 300).await;

    orch.on_turn(turn(Speaker::Interviewer, "вопрос один"));
    let mut rx = orch.subscribe();
    orch.manual(None, None);
    tokio::time::sleep(Duration::from_millis(50)).await;
    orch.manual(None, None);

    let events = timeout(Duration::from_secs(5), async {
        let mut list = 0u32;
        while let Ok(ev) = rx.recv().await {
            match ev {
                OrchEvent::Done { .. } => break,
                OrchEvent::Error { message, .. } => panic!("error: {message}"),
                OrchEvent::Token { .. } => list += 1,
                OrchEvent::Status { .. } => {}
            }
        }
        list
    })
    .await
    .expect("timed out");

    assert!(count.load(Ordering::SeqCst) >= 1);
    assert!(events > 0);
}

#[tokio::test]
async fn turns_accumulate_no_fire() {
    let (orch, count) = orch(mock::sse_body(&["x"]), 0).await;

    orch.on_turn(turn(
        Speaker::Interviewer,
        "как работает event loop в node js и почему это важно",
    ));
    orch.on_turn(turn(Speaker::Candidate, "ну, в общем, там есть цикл"));
    orch.on_turn(turn(
        Speaker::Interviewer,
        "расскажите подробнее про этот цикл",
    ));

    tokio::time::sleep(Duration::from_millis(1000)).await;
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn manual_includes_context() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let (orch, count) = orch_capture(mock::sse_body(&["Привет"]), body_out.clone()).await;

    orch.on_turn(turn(Speaker::Interviewer, "вопрос один"));
    orch.on_turn(turn(Speaker::Candidate, "ответ один"));
    let mut rx = orch.subscribe();
    orch.manual(None, None);

    let result = timeout(Duration::from_secs(5), async {
        let mut done = false;
        while let Ok(ev) = rx.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                done = true;
                break;
            }
        }
        done
    })
    .await
    .expect("timed out");

    assert!(result);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    let body = body_out.lock().unwrap().clone();
    assert!(body.contains("вопрос один"));
    assert!(body.contains("ответ один"));
    assert!(body.contains("вопрос один"));
}

#[tokio::test]
async fn manual_uses_live_partial() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let (orch, count) = orch_capture(mock::sse_body(&["Привет"]), body_out.clone()).await;

    orch.on_partial("а вот градиент для этой функции будет".into());
    let mut rx = orch.subscribe();
    orch.manual(None, None);

    let result = timeout(Duration::from_secs(5), async {
        let mut done = false;
        while let Ok(ev) = rx.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                done = true;
                break;
            }
        }
        done
    })
    .await
    .expect("timed out");

    assert!(result);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    let body = body_out.lock().unwrap().clone();
    assert!(body.contains("а вот градиент для этой функции будет"));
    assert!(body.contains("(ещё говорит)"));
}

#[tokio::test]
async fn vision_payload_sent() {    let body_out = Arc::new(Mutex::new(String::new()));
    let (orch, count) = orch_capture(mock::sse_body(&["Привет"]), body_out.clone()).await;

    let mut rx = orch.subscribe();
    orch.manual(None, Some("QUJD".into()));

    let result = timeout(Duration::from_secs(5), async {
        let mut done = false;
        while let Ok(ev) = rx.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                done = true;
                break;
            }
        }
        done
    })
    .await
    .expect("timed out");

    assert!(result);
    assert_eq!(count.load(Ordering::SeqCst), 1);
    let body = body_out.lock().unwrap().clone();
    assert!(body.contains("image_url"));
    assert!(body.contains("data:image/png;base64,QUJD"));
}

#[test]
fn source_gate() {
    assert!(!gate("manual", true, "system", true));
    assert!(gate("manual", true, "system", false));
    assert!(!gate("manual", true, "mic", false));
    assert!(gate("manual", true, "mic", true));
    assert!(gate("manual", true, "system+mic", true));
    assert!(gate("manual", true, "system+mic", false));
}

#[test]
fn manual_mode_gate() {
    assert!(gate("manual", true, "system+mic", true));
    assert!(!gate("manual", false, "system+mic", true));
}

#[test]
fn vad_mode_gate() {
    assert!(gate("vad", false, "system+mic", true));
    assert!(gate("vad", true, "system+mic", true));
    assert!(!gate("vad", true, "mic", false));
}

#[tokio::test]
async fn chats_isolated() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let (orch, _count) = orch_capture(mock::sse_body(&["Привет"]), body_out.clone()).await;

    orch.on_turn(turn(Speaker::Interviewer, "вопрос в чате 1"));
    orch.on_turn(turn(Speaker::Candidate, "ответ в чате 1"));
    orch.set_active_chat("chat-2".into());
    orch.on_turn(turn(Speaker::Interviewer, "вопрос в чате 2"));
    let mut rx = orch.subscribe();
    orch.manual(None, None);

    let result = timeout(Duration::from_secs(5), async {
        let mut done = false;
        while let Ok(ev) = rx.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                done = true;
                break;
            }
        }
        done
    })
    .await
    .expect("timed out");

    assert!(result);
    let body = body_out.lock().unwrap().clone();
    assert!(body.contains("вопрос в чате 2"));
    assert!(!body.contains("вопрос в чате 1"));
    assert!(!body.contains("ответ в чате 1"));
}

#[tokio::test]
async fn active_chat_gets_turns() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let (orch, _count) = orch_capture(mock::sse_body(&["Привет"]), body_out.clone()).await;

    orch.set_active_chat("chat-2".into());
    orch.on_turn(turn(Speaker::Interviewer, "вопрос в чате 2"));
    orch.on_turn(turn(Speaker::Candidate, "ответ в чате 2"));
    let mut rx = orch.subscribe();
    orch.manual(None, None);

    let result = timeout(Duration::from_secs(5), async {
        let mut done = false;
        while let Ok(ev) = rx.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                done = true;
                break;
            }
        }
        done
    })
    .await
    .expect("timed out");

    assert!(result);
    let body = body_out.lock().unwrap().clone();
    assert!(body.contains("вопрос в чате 2"));
    assert!(body.contains("ответ в чате 2"));
}

#[tokio::test]
async fn reset_active_clears() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let (orch, _count) = orch_capture(mock::sse_body(&["Привет"]), body_out.clone()).await;

    orch.on_turn(turn(Speaker::Interviewer, "вопрос один"));
    orch.on_turn(turn(Speaker::Candidate, "ответ один"));
    orch.on_turn(turn(Speaker::Interviewer, "вопрос два"));
    orch.reset_active();
    orch.on_turn(turn(Speaker::Interviewer, "новый вопрос"));
    let mut rx = orch.subscribe();
    orch.manual(None, None);

    let _result = timeout(Duration::from_secs(5), async {
        let mut done = false;
        while let Ok(ev) = rx.recv().await {
            match ev {
                OrchEvent::Done { .. } => {
                    done = true;
                    break;
                }
                OrchEvent::Error { .. } => {}
                _ => {}
            }
        }
        done
    })
    .await
    .expect("timed out");

    let body = body_out.lock().unwrap().clone();
    assert!(body.contains("новый вопрос"));
    assert!(!body.contains("вопрос один"));
    assert!(!body.contains("ответ один"));
    assert!(!body.contains("вопрос два"));
}

#[tokio::test]
async fn manual_context_persists_across_questions() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let (orch, _count) = orch_capture(mock::sse_body(&["Ответ на первый вопрос"]), body_out.clone()).await;

    // первый вопрос
    let mut rx = orch.subscribe();
    orch.manual(Some("что такое ml".into()), None);
    timeout(Duration::from_secs(5), async {
        while let Ok(ev) = rx.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                break;
            }
        }
    })
    .await
    .expect("timed out");

    // второй вопрос — должен видеть первый Q&A в контексте
    let mut rx2 = orch.subscribe();
    orch.manual(Some("какие виды регуляризации".into()), None);
    timeout(Duration::from_secs(5), async {
        while let Ok(ev) = rx2.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                break;
            }
        }
    })
    .await
    .expect("timed out");

    let body = body_out.lock().unwrap().clone();
    assert!(body.contains("что такое ml"));
    assert!(body.contains("Ответ на первый вопрос"));
    assert!(body.contains("какие виды регуляризации"));
}

#[tokio::test]
async fn auto_toggle_fires() {
    let (orch, count) = orch(mock::sse_body(&["Привет"]), 0).await;
    orch.set_auto(true);
    let mut rx = orch.subscribe();
    orch.on_turn(turn(Speaker::Interviewer, "вопрос автоматически"));

    let result = timeout(Duration::from_secs(5), async {
        let mut done = false;
        while let Ok(ev) = rx.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                done = true;
                break;
            }
        }
        done
    })
    .await
    .expect("timed out");

    assert!(result);
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

// ── 028: long-context memory ─────────────────────────────

use engine_orchestrator::is_key_turn;

#[test]
fn key_question_detected() {
    let t = turn(Speaker::Interviewer, "Объясните, как работает event loop");
    assert!(is_key_turn(&t));
    let t = turn(Speaker::Interviewer, "напиши функцию сортировки");
    assert!(is_key_turn(&t));
    let t = turn(Speaker::Interviewer, "почему выбран этот подход");
    assert!(is_key_turn(&t));
    let long = "х".repeat(201);
    let t = turn(Speaker::Candidate, &long);
    assert!(is_key_turn(&t));
}

#[test]
fn short_not_key() {
    let t = turn(Speaker::Candidate, "да, понял");
    assert!(!is_key_turn(&t));
    let t = turn(Speaker::Interviewer, "хорошо");
    assert!(!is_key_turn(&t));
}

async fn orch_memory_capture(
    sse: String,
    json: String,
    body_out: Arc<Mutex<String>>,
    recent_window: usize,
    key_cap: usize,
) -> Orchestrator {
    let (url, _count) = mock::spawn_mock_auto(sse, json, body_out).await;
    let llm = LlmClient::new(url, "k".into(), "m".into(), 0.0, 100, None).unwrap();
    let ctx = ContextBuilder::new("SYS".into(), "persona".into(), 8000);
    Orchestrator::new(ctx, llm, false).with_memory(recent_window, key_cap, 300, String::new())
}

#[tokio::test]
async fn recent_window_drain_and_summary_updates() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let orch = orch_memory_capture(
        mock::sse_body(&["ok"]),
        "RES".into(),
        body_out.clone(),
        2,
        12,
    )
    .await;

    for i in 1..=5 {
        orch.on_turn(turn(
            Speaker::Interviewer,
            &format!("объясни вопрос номер {i}"),
        ));
    }
    // ждём асинхронную суммаризацию
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut rx = orch.subscribe();
    orch.manual(None, None);
    timeout(Duration::from_secs(5), async {
        while let Ok(ev) = rx.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                break;
            }
        }
    })
    .await
    .expect("timed out");

    let body = body_out.lock().unwrap().clone();
    // summary обновился из мока
    assert!(body.contains("Резюме всей сессии: RES"));
    // слои в правильном порядке
    let pos_summary = body.find("Резюме всей сессии").unwrap();
    let pos_keys = body.find("Ключевые моменты").unwrap();
    let pos_recent = body.find("Недавние реплики").unwrap();
    assert!(pos_summary < pos_keys && pos_keys < pos_recent);
    // ключевые моменты содержат вытесненные реплики (они все «ключевые»)
    let keys_section = &body[pos_keys..pos_recent];
    assert!(keys_section.contains("вопрос номер 1"));
    assert!(keys_section.contains("вопрос номер 3"));
    // окно: только 2 последние реплики в recent
    let recent_section = &body[pos_recent..];
    assert!(recent_section.contains("вопрос номер 5"));
    assert!(recent_section.contains("вопрос номер 4"));
    assert!(!recent_section.contains("вопрос номер 1"));
    assert!(!recent_section.contains("вопрос номер 3"));
}

#[tokio::test]
async fn key_turns_cap_fifo() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let orch = orch_memory_capture(
        mock::sse_body(&["ok"]),
        "RES".into(),
        body_out.clone(),
        50,
        1,
    )
    .await;

    orch.on_turn(turn(Speaker::Interviewer, "объясни альфа подробнее"));
    orch.on_turn(turn(Speaker::Interviewer, "объясни бета подробнее"));
    orch.on_turn(turn(Speaker::Interviewer, "объясни гамма подробнее"));
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut rx = orch.subscribe();
    orch.manual(None, None);
    timeout(Duration::from_secs(5), async {
        while let Ok(ev) = rx.recv().await {
            if matches!(ev, OrchEvent::Done { .. }) {
                break;
            }
        }
    })
    .await
    .expect("timed out");

    let body = body_out.lock().unwrap().clone();
    let start = body.find("Ключевые моменты").expect("no key section");
    let end = body.find("Недавние реплики").expect("no recent section");
    let section = &body[start..end];
    assert!(section.contains("гамма"));
    assert!(!section.contains("альфа"));
    assert!(!section.contains("бета"));
}
