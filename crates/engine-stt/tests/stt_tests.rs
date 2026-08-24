use engine_stt::{
    AudioSegment, CircuitBreaker, CircuitState, DeepgramClient, GroqClient, SttClient, SttError,
    SttQueue,
};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

struct MockServer {
    addr: String,
    requests: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
}

impl MockServer {
    fn spawn(handler: impl Fn(usize) -> (u16, String) + Send + Sync + 'static) -> Self {
        Self::spawn_inner(handler, |_| {})
    }

    fn spawn_capture(
        handler: impl Fn(usize) -> (u16, String) + Send + Sync + 'static,
        capture: impl Fn(Vec<u8>) + Send + Sync + 'static,
    ) -> Self {
        Self::spawn_inner(handler, capture)
    }

    fn spawn_inner(
        handler: impl Fn(usize) -> (u16, String) + Send + Sync + 'static,
        capture: impl Fn(Vec<u8>) + Send + Sync + 'static,
    ) -> Self {
        let handler = Arc::new(handler);
        let capture = Arc::new(capture);
        let requests = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));

        let (tx, rx) = std::sync::mpsc::channel();
        let rt = tokio::runtime::Handle::current();

        let requests_inner = requests.clone();
        let active_inner = active.clone();
        let max_active_inner = max_active.clone();

        std::thread::spawn(move || {
            rt.block_on(async move {
                let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let addr = listener.local_addr().unwrap();
                tx.send(addr.to_string()).unwrap();

                loop {
                    let (mut socket, _) = match listener.accept().await {
                        Ok(v) => v,
                        Err(_) => break,
                    };

                    let requests = requests_inner.clone();
                    let active = active_inner.clone();
                    let max_active = max_active_inner.clone();
                    let handler = handler.clone();
                    let capture = capture.clone();

                    tokio::spawn(async move {
                        let active_now = active.fetch_add(1, Ordering::SeqCst) + 1;
                        max_active.fetch_max(active_now, Ordering::SeqCst);

                        let request_id = requests.fetch_add(1, Ordering::SeqCst);
                        let (status, body) = handler(request_id);

                        let req_body = read_http_request(&mut socket).await.unwrap_or_default();
                        capture(req_body);

                        let mut response = Vec::new();
                        let reason = if status == 200 { "OK" } else { "Error" };
                        write!(
                            response,
                            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        )
                        .unwrap();

                        let _ = socket.write_all(&response).await;
                        let _ = socket.shutdown().await;

                        active.fetch_sub(1, Ordering::SeqCst);
                    });
                }
            });
        });

        let addr = rx.recv().unwrap();

        Self {
            addr,
            requests,
            max_active,
        }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }
}

async fn read_http_request(socket: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut buf = vec![0u8; 262144];
    let n = socket.read(&mut buf).await?;
    buf.truncate(n);
    Ok(buf)
}

fn ok_transcript(text: &str) -> (u16, String) {
    let body = format!(r#"{{"text": "{}", "duration": 1.0}}"#, text);
    (200, body)
}

fn audio_segment() -> AudioSegment {
    AudioSegment {
        id: uuid::Uuid::new_v4(),
        audio: vec![0.0f32; 16000],
        duration_ms: 1000,
    }
}

fn client(base_url: &str) -> GroqClient {
    GroqClient::with_base_url("test-key".to_string(), base_url.to_string()).unwrap()
}

fn client_arc(base_url: &str) -> Arc<SttClient> {
    Arc::new(SttClient::Groq(client(base_url)))
}

fn deepgram_client(base_url: &str) -> DeepgramClient {
    DeepgramClient::with_base_url("test-key".to_string(), "nova-3".to_string(), base_url.to_string())
        .unwrap()
}

fn deepgram_ok_transcript(text: &str) -> (u16, String) {
    let body = format!(
        r#"{{"metadata": {{"duration": 1.0}}, "results": {{"channels": [{{"alternatives": [{{"transcript": "{}", "confidence": 0.98, "words": [{{"confidence": 0.97}}]}}]}}]}}}}"#,
        text
    );
    (200, body)
}

#[tokio::test]
async fn transcribes_successfully() {
    let server = MockServer::spawn(|_req| ok_transcript("hello world"));

    let client = client(&server.url());
    let transcript = client.transcribe(&vec![0.0f32; 16000]).await.unwrap();
    assert_eq!(transcript.text, "hello world");
    assert!(transcript.duration > 0.0);
}

#[tokio::test]
async fn errors_on_invalid_key() {
    let server = MockServer::spawn(|_req| (401, r#"{"error": "invalid key"}"#.to_string()));

    let client = client(&server.url());
    let result = client.transcribe(&vec![0.0f32; 16000]).await;
    assert!(matches!(result, Err(SttError::Authentication)));
}

#[tokio::test]
async fn stt_language_sent() {
    let captured = Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let captured_inner = captured.clone();
    let server = MockServer::spawn_capture(
        |_req| ok_transcript("привет"),
        move |body| {
            *captured_inner.lock().unwrap() = body;
        },
    );

    let client = client(&server.url()).with_language("ru".into());
    let transcript = client.transcribe(&vec![0.0f32; 16000]).await.unwrap();
    assert_eq!(transcript.text, "привет");

    let raw = String::from_utf8_lossy(&captured.lock().unwrap().clone()).to_string();
    assert!(
        raw.contains("language"),
        "expected language field in multipart, got: {raw}"
    );
}

#[tokio::test]
async fn respects_concurrency_limit() {
    let server = MockServer::spawn(|_req| {
        std::thread::sleep(Duration::from_millis(100));
        ok_transcript("ok")
    });

    let client = client_arc(&server.url());
    let (queue, mut receiver) = SttQueue::new(client, 3, 100);

    for _ in 0..10 {
        queue.submit(audio_segment()).await.unwrap();
    }

    let mut received = 0;
    while let Ok(Some((_segment, result))) =
        tokio::time::timeout(Duration::from_secs(10), receiver.recv()).await
    {
        assert!(result.is_ok());
        received += 1;
        if received == 10 {
            break;
        }
    }
    assert_eq!(received, 10);
    assert!(server.max_active.load(Ordering::SeqCst) <= 3);
}

#[tokio::test]
async fn rejects_on_overflow() {
    let server = MockServer::spawn(|_req| {
        std::thread::sleep(Duration::from_millis(200));
        ok_transcript("ok")
    });

    let client = client_arc(&server.url());
    let (queue, mut receiver) = SttQueue::new(client, 1, 2);

    queue.submit(audio_segment()).await.unwrap();
    queue.submit(audio_segment()).await.unwrap();

    let result = queue.submit(audio_segment()).await;
    assert!(matches!(result, Err(SttError::QueueFull)));

    while let Ok(Some(_)) = tokio::time::timeout(Duration::from_secs(5), receiver.recv()).await {}
}

#[tokio::test]
async fn retries_on_transient_error() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();
    let server = MockServer::spawn(move |_req| {
        let n = attempts_clone.fetch_add(1, Ordering::SeqCst);
        if n == 0 {
            (500, "server error".to_string())
        } else {
            ok_transcript("retried")
        }
    });

    let client = client_arc(&server.url());
    let (queue, mut receiver) = SttQueue::new(client, 1, 10);

    queue.submit(audio_segment()).await.unwrap();

    let (_, result) = tokio::time::timeout(Duration::from_secs(5), receiver.recv())
        .await
        .expect("timeout")
        .expect("channel closed");
    let transcript = result.unwrap();
    assert_eq!(transcript.text, "retried");
    assert!(attempts.load(Ordering::SeqCst) >= 2);
}

#[tokio::test]
async fn fails_after_max_retries() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();
    let server = MockServer::spawn(move |_req| {
        attempts_clone.fetch_add(1, Ordering::SeqCst);
        (500, "server error".to_string())
    });

    let client = client_arc(&server.url());
    let (queue, mut receiver) = SttQueue::new(client, 1, 10);

    queue.submit(audio_segment()).await.unwrap();

    let (_, result) = tokio::time::timeout(Duration::from_secs(5), receiver.recv())
        .await
        .expect("timeout")
        .expect("channel closed");
    assert!(matches!(result, Err(SttError::MaxRetriesExceeded { .. })));
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn opens_circuit_on_failures() {
    let breaker = CircuitBreaker::new(5, 30);

    for _ in 0..5 {
        assert!(breaker.allow_request().await);
        breaker.record_failure().await;
    }

    assert_eq!(breaker.current_state(), CircuitState::Open);
    assert!(!breaker.allow_request().await);
}

#[tokio::test]
async fn closes_circuit_on_success() {
    let breaker = CircuitBreaker::new(2, 1);

    for _ in 0..2 {
        breaker.record_failure().await;
    }
    assert_eq!(breaker.current_state(), CircuitState::Open);
    assert!(!breaker.allow_request().await);

    sleep(Duration::from_millis(1100)).await;
    assert!(breaker.allow_request().await);
    assert_eq!(breaker.current_state(), CircuitState::HalfOpen);

    breaker.record_success().await;
    assert_eq!(breaker.current_state(), CircuitState::Closed);
}

#[tokio::test]
async fn streams_transcripts() {
    let server = MockServer::spawn(|req| ok_transcript(&format!("segment {req}")));

    let client = client_arc(&server.url());
    let (queue, mut receiver) = SttQueue::new(client, 3, 100);

    for i in 0..5 {
        let mut segment = audio_segment();
        segment.id = uuid::Uuid::from_u128(i as u128);
        queue.submit(segment).await.unwrap();
    }

    let mut received = Vec::new();
    while let Ok(Some((segment, result))) =
        tokio::time::timeout(Duration::from_secs(5), receiver.recv()).await
    {
        let transcript = result.unwrap();
        received.push((segment.id, transcript.text));
        if received.len() == 5 {
            break;
        }
    }

    received.sort_by_key(|(id, _)| *id);
    for (i, (_, text)) in received.iter().enumerate() {
        assert_eq!(text, &format!("segment {i}"));
    }
}

#[tokio::test]
async fn circuit_blocks_when_open() {
    let server = MockServer::spawn(|_req| (500, "error".to_string()));

    let client = client_arc(&server.url());
    let (queue, mut receiver) = SttQueue::new(client, 1, 10);

    for _ in 0..5 {
        queue.submit(audio_segment()).await.unwrap();
    }

    let mut failures = 0;
    while let Ok(Some((_, result))) =
        tokio::time::timeout(Duration::from_secs(5), receiver.recv()).await
    {
        assert!(result.is_err());
        failures += 1;
        if failures >= 5 {
            break;
        }
    }

    let server_requests_before = server.requests.load(Ordering::SeqCst);
    queue.submit(audio_segment()).await.unwrap();
    let (_, result) = tokio::time::timeout(Duration::from_secs(5), receiver.recv())
        .await
        .expect("timeout")
        .expect("channel closed");
    assert!(matches!(result, Err(SttError::CircuitOpen)));

    let server_requests_after = server.requests.load(Ordering::SeqCst);
    assert_eq!(server_requests_before, server_requests_after);
}

#[tokio::test]
async fn deepgram_transcribes_successfully() {
    let server = MockServer::spawn(|_req| deepgram_ok_transcript("привет мир"));

    let client = deepgram_client(&server.url());
    let transcript = client.transcribe(&vec![0.0f32; 16000]).await.unwrap();
    assert_eq!(transcript.text, "привет мир");
    assert!(transcript.avg_logprob < 0.0);
}

#[tokio::test]
async fn deepgram_uses_model_param() {
    let captured = Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let captured_inner = captured.clone();
    let server = MockServer::spawn_capture(
        |_req| deepgram_ok_transcript("ok"),
        move |body| {
            *captured_inner.lock().unwrap() = body;
        },
    );

    let client = deepgram_client(&server.url());
    client.transcribe(&vec![0.0f32; 16000]).await.unwrap();

    let raw = String::from_utf8_lossy(&captured.lock().unwrap().clone()).to_string();
    assert!(
        raw.contains("model=nova-3"),
        "expected model=nova-3 in request, got: {raw}"
    );
}

#[tokio::test]
async fn deepgram_invalid_key() {
    let server = MockServer::spawn(|_req| (401, r#"{"err_code": "INVALID_API_KEY"}"#.to_string()));

    let client = deepgram_client(&server.url());
    let result = client.transcribe(&vec![0.0f32; 16000]).await;
    assert!(matches!(result, Err(SttError::Authentication)));
}
