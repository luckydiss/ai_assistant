use engine_rtstt::{render, RtEvent, SonioxConfig, CHUNK};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio::net::TcpListener as TokioListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

async fn start_mock() -> TokioListener {
    TokioListener::bind("127.0.0.1:0").await.unwrap()
}

fn rt_config(listener: &TokioListener) -> SonioxConfig {
    let addr = listener.local_addr().unwrap();
    let mut cfg = SonioxConfig::new("test_key".into(), "stt-rt-v5".into());
    cfg.ws_url = format!("ws://{addr}/transcribe-websocket");
    cfg
}

async fn next_event(rx: &mut tokio::sync::mpsc::Receiver<RtEvent>) -> RtEvent {
    tokio::time::timeout(Duration::from_secs(3), rx.recv())
        .await
        .expect("timeout waiting for event")
        .unwrap()
}

async fn next_utterance(rx: &mut tokio::sync::mpsc::Receiver<RtEvent>) -> RtEvent {
    loop {
        match next_event(rx).await {
            ev @ RtEvent::Utterance { .. } => return ev,
            ev @ RtEvent::Error(_) => return ev,
            RtEvent::Partial { .. } | RtEvent::Closed => continue,
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn first_message_is_config() {
    let listener = start_mock().await;
    let mut cfg = rt_config(&listener);
    cfg.language_hints = vec!["ru".into(), "en".into()];

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let first = ws.next().await.unwrap().unwrap();
        let Message::Text(txt) = first else {
            panic!("first frame not text");
        };
        let v: serde_json::Value = serde_json::from_str(&txt).unwrap();
        assert_eq!(v["api_key"], "test_key");
        assert_eq!(v["sample_rate"], 16000);
        assert_eq!(v["num_channels"], 1);
        assert_eq!(v["audio_format"], "pcm_s16le");
        assert_eq!(v["model"], "stt-rt-v5");
        assert_eq!(v["enable_endpoint_detection"], false);
        assert_eq!(v["language_hints"], json!(["ru", "en"]));
    });

    let _session = engine_rtstt::start(cfg).unwrap();
    server.await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn chunk_size_3840() {
    let listener = start_mock().await;
    let cfg = rt_config(&listener);
    let mut session = engine_rtstt::start(cfg).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap(); // config
        let mut sizes = Vec::new();
        while let Some(msg) = ws.next().await {
            let msg = msg.unwrap();
            match msg {
                Message::Binary(b) => sizes.push(b.len()),
                Message::Text(t) if t.is_empty() => break,
                _ => {}
            }
        }
        assert_eq!(sizes, vec![CHUNK, CHUNK]);
    });

    let audio = vec![0i16; CHUNK / 2]; // CHUNK/2 сэмплов = CHUNK байт
    session.audio_tx.send(to_bytes(&audio)).await.unwrap();
    session.audio_tx.send(to_bytes(&audio)).await.unwrap();
    drop(session.audio_tx);
    let _ = session.events.recv().await;
    server.await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn partial_renders_finals_plus_nonfinals() {
    let listener = start_mock().await;
    let cfg = rt_config(&listener);
    let mut session = engine_rtstt::start(cfg).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap(); // config
        let msg = json!({
            "tokens": [
                { "text": "Как ", "is_final": true },
                { "text": "работает", "is_final": false }
            ]
        });
        ws.send(Message::Text(msg.to_string())).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let _ = ws.close(None).await;
    });

    let ev = tokio::time::timeout(Duration::from_secs(3), session.events.recv())
        .await
        .unwrap()
        .unwrap();
    match ev {
        RtEvent::Partial { text, .. } => assert_eq!(text, "Как работает"),
        other => panic!("unexpected event: {other:?}"),
    }
    server.await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn utterance_on_token_silence() {
    let listener = start_mock().await;
    let cfg = rt_config(&listener);
    let mut session = engine_rtstt::start(cfg).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap(); // config
        let msg = json!({ "tokens": [{ "text": "Привет мир", "is_final": true }] });
        ws.send(Message::Text(msg.to_string())).await.unwrap();
        tokio::time::sleep(Duration::from_millis(1500)).await;
    });

    let ev = next_utterance(&mut session.events).await;
    match ev {
        RtEvent::Utterance { text, .. } => assert_eq!(text, "Привет мир"),
        other => panic!("unexpected event: {other:?}"),
    }
    server.await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn utterance_on_finished() {
    let listener = start_mock().await;
    let cfg = rt_config(&listener);
    let mut session = engine_rtstt::start(cfg).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap(); // config
        let msg = json!({
            "tokens": [{ "text": "Всё готово", "is_final": true }],
            "finished": true
        });
        ws.send(Message::Text(msg.to_string())).await.unwrap();
    });

    let ev = next_utterance(&mut session.events).await;
    match ev {
        RtEvent::Utterance { text, .. } => assert_eq!(text, "Всё готово"),
        other => panic!("unexpected event: {other:?}"),
    }
    server.await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn error_event() {
    let listener = start_mock().await;
    let cfg = rt_config(&listener);
    let mut session = engine_rtstt::start(cfg).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap(); // config
        let msg = json!({
            "error_code": "auth_failed",
            "error_message": "invalid api key"
        });
        ws.send(Message::Text(msg.to_string())).await.unwrap();
    });

    let ev = tokio::time::timeout(Duration::from_secs(3), session.events.recv())
        .await
        .unwrap()
        .unwrap();
    match ev {
        RtEvent::Error(msg) => assert!(msg.contains("auth_failed")),
        other => panic!("unexpected event: {other:?}"),
    }
    server.await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn lanes_isolated() {
    let listener1 = start_mock().await;
    let listener2 = start_mock().await;
    let cfg1 = rt_config(&listener1);
    let cfg2 = rt_config(&listener2);

    let server1 = tokio::spawn(async move {
        let (stream, _) = listener1.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap();
        let msg = json!({ "tokens": [{ "text": "из I", "is_final": true }], "finished": true });
        ws.send(Message::Text(msg.to_string())).await.unwrap();
    });
    let server2 = tokio::spawn(async move {
        let (stream, _) = listener2.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap();
        let msg = json!({ "tokens": [{ "text": "из C", "is_final": true }], "finished": true });
        ws.send(Message::Text(msg.to_string())).await.unwrap();
    });

    let mut s1 = engine_rtstt::start(cfg1).unwrap();
    let mut s2 = engine_rtstt::start(cfg2).unwrap();
    let e1 = next_utterance(&mut s1.events).await;
    let e2 = next_utterance(&mut s2.events).await;
    assert!(matches!(e1, RtEvent::Utterance { text: ref t, .. } if t == "из I"));
    assert!(matches!(e2, RtEvent::Utterance { text: ref t, .. } if t == "из C"));
    server1.await.unwrap();
    server2.await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn closed_on_ws_drop() {
    let listener = start_mock().await;
    let cfg = rt_config(&listener);
    let mut session = engine_rtstt::start(cfg).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();
        let _ = ws.next().await.unwrap().unwrap(); // config
        // сразу закрываем
        let _ = ws.close(None).await;
    });

    let ev = tokio::time::timeout(Duration::from_secs(3), session.events.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(ev, RtEvent::Closed), "expected Closed, got {ev:?}");
    server.await.unwrap();
}

#[test]
fn render_concats_finals_and_nonfinal() {
    let finals = vec!["Как ".into(), "работает ".into()];
    assert_eq!(render(&finals, "сейчас"), "Как работает сейчас");
}

#[test]
fn render_joins_subword_tokens_without_extra_spaces() {
    let finals = vec![
        "Ко".into(),
        "не".into(),
        "ц".into(),
        " ".into(),
        "ст".into(),
        "рои".into(),
        "лся".into(),
    ];
    assert_eq!(render(&finals, ""), "Конец строился");
}

#[test]
fn render_collapses_newlines_into_space() {
    let finals = vec!["комбинация\nмежду".into(), " ".into(), "precision\nи\nrecall".into()];
    assert_eq!(render(&finals, ""), "комбинация между precision и recall");
}

fn to_bytes(samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}