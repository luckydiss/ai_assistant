# Project: Interview Assistant (Sobes Analog)

## Toolchain
- **Rust:** 1.75+ (edition 2021)
- **Target:** Windows 10/11 x64 only
- **UI Framework:** Tauri v2 (WebView2 on Windows)

## Workspace Structure
```
interview-assistant/
├── Cargo.toml (workspace)
├── apps/
│   └── desktop/           (Tauri app)
├── crates/
│   ├── engine-config/     (config + hot-reload)
│   ├── engine-audio/      (cpal + WASAPI)
│   ├── engine-vad/        (Silero VAD)
│   ├── engine-stt/        (Groq client)
│   ├── engine-dialogue/   (assembler + reorder)
│   ├── engine-orchestrator/ (state machine)
│   ├── engine-context/    (LLM prompt builder)
│   ├── engine-llm/        (OpenAI-compatible SSE)
│   ├── engine-store/      (sqlite)
│   └── engine-ipc/        (Tauri events)
└── .opencode/
```

## Dependency Matrix (pinned versions)
```toml
[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
cpal = { version = "0.15", features = ["wasapi"] }
ort = { version = "2.0", features = ["load-dynamic"] }
reqwest = { version = "0.11", features = ["json", "stream", "rustls-tls"] }
tauri = { version = "2.0", features = [] }
rusqlite = { version = "0.31", features = ["bundled"] }
notify = "6.1"
tokio-stream = "0.1"
futures = "0.3"
uuid = { version = "1.6", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
windows = { version = "0.52", features = ["Win32_UI_WindowsAndMessaging", "Win32_Graphics_Dxgi"] }
```

## Forbidden List (DO NOT add these)
- Any macOS/Linux-specific crates
- Async-std (use tokio only)
- Any audio processing crates not listed
- Any ML frameworks besides ort
- Actix-warp-hyper (use reqwest only)

## Coding Policies
- **Errors:** `thiserror` in libs, `anyhow` in binary
- **Logging:** `tracing` macros only (not println)
- **Tests:** every spec scenario = 1 test with matching name
- **Clippy:** `#![deny(clippy::all)]` in every crate
- **No unwrap()** in production code, use Result/Option properly

## Platform Specifics
- **Audio capture:** WASAPI loopback for system audio
- **Invisibility:** `SetWindowDisplayAffinity` with `WDA_EXCLUDEFROMCAPTURE`
- **Self-check:** DXGI Desktop Duplication API
- **UI:** WebView2 transparency (potential artifacts, fallback: semi-transparent)
