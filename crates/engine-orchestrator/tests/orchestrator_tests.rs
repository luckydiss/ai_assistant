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
