# Design: Bootstrap

## 1. Workspace Cargo.toml

Полный файл workspace root:

```toml
[workspace]
resolver = "2"
members = [
    "apps/desktop",
    "crates/engine-config",
    "crates/engine-audio",
    "crates/engine-vad",
    "crates/engine-stt",
    "crates/engine-dialogue",
    "crates/engine-orchestrator",
    "crates/engine-context",
    "crates/engine-llm",
    "crates/engine-store",
    "crates/engine-ipc",
]

[workspace.package]
edition = "2021"
rust-version = "1.75"

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

## 2. Crate Shell Pattern

Каждый crate в `crates/` имеет структуру:

```
crates/engine-{name}/
├── Cargo.toml
└── src/
    └── lib.rs
```

**Cargo.toml template:**

```toml
[package]
name = "engine-{name}"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true
```

**src/lib.rs template:**

```rust
//! Engine {name} module
#![deny(clippy::all)]
```

## 3. Desktop App Shell

`apps/desktop/` — это Tauri приложение.

**Cargo.toml:**

```toml
[package]
name = "desktop"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
tauri.workspace = true
tokio.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

[build-dependencies]
tauri-build = "2.0"
```

**src/main.rs:**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    tauri::Builder::default()
        .setup(|_app| {
            tracing::info!("Desktop app started");
            Ok(())
        })
        .run(tauri::generate_context!())?;

    Ok(())
}
```

**build.rs:**

```rust
fn main() {
    tauri_build::build()
}
```

**src-tauri/tauri.conf.json:**

```json
{
  "productName": "Interview Assistant",
  "version": "0.1.0",
  "identifier": "com.interview.assistant",
  "build": {
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Interview Assistant",
        "width": 800,
        "height": 600
      }
    ]
  }
}
```

## 4. GitHub Actions CI

`.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: 1.75
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo build --workspace
      - run: cargo test --workspace
```

## 5. Root .gitignore

```gitignore
/target/
**/*.rs.bk
Cargo.lock.bak
.idea/
.vscode/
*.swp
.DS_Store
node_modules/
dist/
```

## 6. Root rustfmt.toml

```toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
```

## 7. Root clippy.toml

```toml
cognitive-complexity-threshold = 25
```

## Рассмотрено и отклонено
- **Async-std:** отклонено, используем tokio (см. project.md)
- **Прямые WinAPI bindings:** отклонено, используем `windows` crate
- **Custom build scripts:** отклонено, используем `tauri-build`
