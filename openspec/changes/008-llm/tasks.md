# Tasks: LLM Client

- [x] 1.1 РћР±РЅРѕРІРёС‚СЊ `crates/engine-llm/Cargo.toml` РїРѕ design.md В§1
  verify: `cargo build -p engine-llm`

- [x] 2.1 РЎРѕР·РґР°С‚СЊ `src/sse.rs` РїРѕ design.md В§3
  verify: `cargo build -p engine-llm`

- [x] 2.2 РЎРѕР·РґР°С‚СЊ `src/skip.rs` РїРѕ design.md В§4
  verify: `cargo build -p engine-llm`

- [x] 2.3 РЎРѕР·РґР°С‚СЊ `src/client.rs` РїРѕ design.md В§5
  verify: `cargo build -p engine-llm`

- [x] 2.4 РћР±РЅРѕРІРёС‚СЊ `src/lib.rs` РїРѕ design.md В§2
  verify: `cargo build -p engine-llm`

- [x] 3.0 РЎРѕР·РґР°С‚СЊ `tests/mock.rs` РїРѕ design.md В§6
  verify: С„Р°Р№Р» СЃСѓС‰РµСЃС‚РІСѓРµС‚

- [x] 3.1 РўРµСЃС‚С‹ `parses_sse_data_lines`, `extracts_delta`
  verify: `cargo test -p engine-llm sse`

- [x] 3.2 РўРµСЃС‚С‹ `skip_detected`, `passthrough_after_partial`
  verify: `cargo test -p engine-llm skip`

- [x] 3.3 РўРµСЃС‚ `streams_tokens_from_mock_server`
  verify: `cargo test -p engine-llm streams_tokens_from_mock_server`

- [x] 3.4 РўРµСЃС‚ `skip_emits_no_tokens`
  verify: `cargo test -p engine-llm skip_emits_no_tokens`

- [x] 3.5 РўРµСЃС‚ `cancel_aborts_stream`
  verify: `cargo test -p engine-llm cancel_aborts_stream`

- [x] 3.6 РўРµСЃС‚ `fails_on_401` (mock РІРѕР·РІСЂР°С‰Р°РµС‚ "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n")
  verify: `cargo test -p engine-llm fails_on_401`

- [x] 4.1 `cargo clippy -p engine-llm --all-targets -- -D warnings`
  verify: РІС‹С…РѕРґ 0

## STOP Protocol
Р•СЃР»Рё reqwest bytes_stream РЅРµ РєРѕРјРїРёР»РёСЂСѓРµС‚СЃСЏ вЂ” РїСЂРѕРІРµСЂРёС‚СЊ feature "stream" РІ workspace-Р·Р°РІРёСЃРёРјРѕСЃС‚Рё reqwest (СѓР¶Рµ РІ project.md).
РќРµ РґРѕР±Р°РІР»СЏС‚СЊ async-openai РёР»Рё eventsource-РєР»РёРµРЅС‚С‹. РћСЃС‚Р°РЅРѕРІРёС‚СЊСЃСЏ Рё СЃРїСЂРѕСЃРёС‚СЊ.

