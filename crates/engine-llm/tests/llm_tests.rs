mod mock;

use engine_llm::{extract_delta, parse_sse_line, AnswerEvent, LlmClient};
use mock::{spawn_mock_response, spawn_mock_sse, spawn_mock_sse_capture, sse_body};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

#[test]
fn parses_sse_data_lines() {
    assert_eq!(parse_sse_line("data: {\"a\":1}"), Some("{\"a\":1}"));
    assert_eq!(parse_sse_line("  data:  x  "), Some("x"));
    assert_eq!(parse_sse_line("event: message"), None);
    assert_eq!(parse_sse_line(""), None);
}

#[test]
fn extracts_delta() {
    assert_eq!(
        extract_delta(r#"{"choices":[{"delta":{"content":"Пр"}}]}"#),
        Some("Пр".to_string())
    );
    assert_eq!(extract_delta(r#"{"choices":[{"delta":{}}]}"#), None);
    assert_eq!(extract_delta("not json"), None);
}

#[tokio::test]
async fn streams_tokens_from_mock_server() {
    let (url, _) = spawn_mock_sse(sse_body(&["Пр", "ивет"]), 0).await;
    let client = LlmClient::new(url, "key".into(), "m".into(), 0.0, 100, None).unwrap();

    let (mut rx, handle) = client.stream_answer(vec![]);

    let result = timeout(Duration::from_secs(5), async {
        let mut tokens = Vec::new();
        let mut done = None;
        while let Some(ev) = rx.recv().await {
            match ev {
                AnswerEvent::Token(t) => tokens.push(t),
                AnswerEvent::Done(full) => {
                    done = Some(full);
                    break;
                }
                AnswerEvent::Failed(e) => panic!("unexpected failed: {e}"),
            }
        }
        (tokens, done)
    })
    .await
    .expect("timed out");

    handle.abort();
    assert_eq!(result.0, vec!["Пр".to_string(), "ивет".to_string()]);
    assert_eq!(result.1, Some("Привет".to_string()));
}

#[tokio::test]
async fn cancel_aborts_stream() {
    let (url, _) = spawn_mock_sse(sse_body(&["slow"]), 500).await;
    let client = LlmClient::new(url, "key".into(), "m".into(), 0.0, 100, None).unwrap();

    let (mut rx, handle) = client.stream_answer(vec![]);
    handle.abort();

    let result = timeout(Duration::from_secs(3), async {
        let mut events = Vec::new();
        while let Some(ev) = rx.recv().await {
            events.push(ev);
        }
        events
    })
    .await
    .expect("receiver did not close");

    assert!(result.is_empty());
}

#[tokio::test]
async fn fails_on_401() {
    let (url, _) = spawn_mock_response("HTTP/1.1 401 Unauthorized", String::new(), 0).await;
    let client = LlmClient::new(url, "key".into(), "m".into(), 0.0, 100, None).unwrap();

    let (mut rx, handle) = client.stream_answer(vec![]);

    let result = timeout(Duration::from_secs(5), async {
        let mut failed = None;
        while let Some(ev) = rx.recv().await {
            if let AnswerEvent::Failed(e) = ev {
                failed = Some(e);
                break;
            }
        }
        failed
    })
    .await
    .expect("timed out")
    .expect("no failed event");

    handle.abort();
    assert!(
        result.contains("auth"),
        "expected auth mention, got {result}"
    );
}

#[tokio::test]
async fn reasoning_effort_sent() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let (url, _) = spawn_mock_sse_capture(sse_body(&["ok"]), body_out.clone()).await;
    let client =
        LlmClient::new(url, "key".into(), "m".into(), 0.0, 100, Some("low".into())).unwrap();

    let (mut rx, handle) = client.stream_answer(vec![]);
    let _ = timeout(Duration::from_secs(5), rx.recv()).await;
    handle.abort();

    let body = body_out.lock().unwrap().clone();
    assert!(
        body.contains("\"reasoning_effort\":\"low\""),
        "expected reasoning_effort in body, got {body}"
    );
}

#[tokio::test]
async fn search_tool_injected() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let (url, _) = spawn_mock_sse_capture(sse_body(&["ok"]), body_out.clone()).await;
    let client = LlmClient::new(url, "key".into(), "m".into(), 0.0, 100, None)
        .unwrap()
        .with_search(true, r#"{"enable_search":true}"#.into());

    let (mut rx, handle) = client.stream_answer(vec![]);
    let _ = timeout(Duration::from_secs(5), rx.recv()).await;
    handle.abort();

    let body = body_out.lock().unwrap().clone();
    assert!(
        body.contains("\"enable_search\":true"),
        "expected enable_search in body, got {body}"
    );
}

#[tokio::test]
async fn search_tool_absent() {
    let body_out = Arc::new(Mutex::new(String::new()));
    let (url, _) = spawn_mock_sse_capture(sse_body(&["ok"]), body_out.clone()).await;
    let client = LlmClient::new(url, "key".into(), "m".into(), 0.0, 100, None).unwrap();

    let (mut rx, handle) = client.stream_answer(vec![]);
    let _ = timeout(Duration::from_secs(5), rx.recv()).await;
    handle.abort();

    let body = body_out.lock().unwrap().clone();
    assert!(
        !body.contains("enable_search"),
        "expected no enable_search in body, got {body}"
    );
}
