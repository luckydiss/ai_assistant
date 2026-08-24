mod mock_ws;

use engine_tts::cartesia::{start_session_at, CartesiaConfig, TtsCmd, TtsOut};

fn cfg() -> CartesiaConfig {
    CartesiaConfig {
        api_key: "sk_test".into(),
        model_id: "sonic-3.5".into(),
        voice_id: "v".into(),
        sample_rate: 22050,
    }
}

#[tokio::test]
async fn first_audio_before_done() {
    let mock = mock_ws::spawn().await.unwrap();
    let url = format!("ws://127.0.0.1:{}", mock.port);
    let mut s = start_session_at(cfg(), url.clone()).unwrap();

    s.cmd.send(TtsCmd::Text("Привет!".into())).await.unwrap();
    let pcm = tokio::time::timeout(std::time::Duration::from_secs(5), s.out.recv())
        .await
        .expect("timeout waiting first chunk")
        .expect("channel closed");
    match pcm {
        TtsOut::Pcm(f) => assert!(!f.is_empty()),
        other => panic!("expected Pcm, got {:?}", variant(&other)),
    }
    s.cmd.send(TtsCmd::Flush).await.unwrap();
    let done = tokio::time::timeout(std::time::Duration::from_secs(5), s.out.recv())
        .await
        .expect("timeout waiting done")
        .expect("channel closed");
    assert!(matches!(done, TtsOut::Done));
}

#[tokio::test]
async fn finalize_sent() {
    let mock = mock_ws::spawn().await.unwrap();
    let url = format!("ws://127.0.0.1:{}", mock.port);
    let mut s = start_session_at(cfg(), url.clone()).unwrap();

    s.cmd.send(TtsCmd::Text("Привет!".into())).await.unwrap();
    s.cmd.send(TtsCmd::Flush).await.unwrap();
    drain_until_done(&mut s).await;
    assert!(mock.flush_seen.load(std::sync::atomic::Ordering::SeqCst));
}

#[tokio::test]
async fn context_per_answer() {
    let mock = mock_ws::spawn().await.unwrap();
    let url = format!("ws://127.0.0.1:{}", mock.port);

    for _ in 0..2 {
        let mut s = start_session_at(cfg(), url.clone()).unwrap();
        s.cmd.send(TtsCmd::Text("Привет!".into())).await.unwrap();
        s.cmd.send(TtsCmd::Flush).await.unwrap();
        drain_until_done(&mut s).await;
    }

    let ids = mock.context_ids.lock().unwrap();
    assert!(ids.len() >= 2, "expected at least 2 text messages");
    let mut uniq: Vec<String> = Vec::new();
    for id in ids.iter() {
        if !uniq.contains(id) {
            uniq.push(id.clone());
        }
    }
    assert_eq!(uniq.len(), 2, "two sessions should use distinct context ids");
    assert_ne!(uniq[0], uniq[1]);
}

async fn drain_until_done(s: &mut engine_tts::cartesia::TtsSession) {
    loop {
        let out = tokio::time::timeout(std::time::Duration::from_secs(5), s.out.recv())
            .await
            .expect("timeout waiting done")
            .expect("channel closed");
        if matches!(out, TtsOut::Done) {
            break;
        }
    }
}

fn variant(_o: &TtsOut) -> &'static str {
    "unexpected"
}