use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub fn sse_body(deltas: &[&str]) -> String {
    let mut s = String::new();
    for d in deltas {
        s.push_str(&format!(
            "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{}\"}}}}]}}\n\n",
            d
        ));
    }
    s.push_str("data: [DONE]\n\n");
    s
}

/// Spawns a mock SSE server that always returns `status_line` and `body`.
pub async fn spawn_mock_response(
    status_line: &str,
    body: String,
    delay_ms: u64,
) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let c2 = count.clone();

    let status_line = status_line.to_string();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let c2 = c2.clone();
            let body = body.clone();
            let status_line = status_line.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 8192];
                let _ = sock.read(&mut buf).await;
                c2.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                let head = format!("{}\r\nContent-Length: {}\r\n\r\n", status_line, body.len());
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(body.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });

    (format!("http://{}", addr), count)
}

pub async fn spawn_mock_sse(body: String, delay_ms: u64) -> (String, Arc<AtomicUsize>) {
    spawn_mock_response(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream",
        body,
        delay_ms,
    )
    .await
}

/// Spawns an SSE mock server that captures the request body into `body_out`.
pub async fn spawn_mock_sse_capture(
    body: String,
    body_out: Arc<Mutex<String>>,
) -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let count = Arc::new(AtomicUsize::new(0));
    let c2 = count.clone();

    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let c2 = c2.clone();
            let body = body.clone();
            let body_out = body_out.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 65536];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                if let Some(idx) = req.find("\r\n\r\n") {
                    let payload = req[idx + 4..].trim().to_string();
                    if let Ok(mut guard) = body_out.lock() {
                        *guard = payload;
                    }
                }
                c2.fetch_add(1, Ordering::SeqCst);
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n",
                    body.len()
                );
                let _ = sock.write_all(head.as_bytes()).await;
                let _ = sock.write_all(body.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });

    (format!("http://{}", addr), count)
}
