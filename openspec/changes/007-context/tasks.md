# Tasks: Context Builder

- [x] 1.1 РћР±РЅРѕРІРёС‚СЊ `crates/engine-context/Cargo.toml` РїРѕ design.md В§1
  verify: `cargo build -p engine-context`

- [x] 2.1 РЎРѕР·РґР°С‚СЊ `src/tokens.rs` РїРѕ design.md В§3
  verify: `cargo build -p engine-context`

- [x] 2.2 РЎРѕР·РґР°С‚СЊ `src/builder.rs` РїРѕ design.md В§4 (Role, ChatMessage, ContextBuilder)
  verify: `cargo build -p engine-context`

- [x] 2.3 РћР±РЅРѕРІРёС‚СЊ `src/lib.rs` РїРѕ design.md В§2
  verify: `cargo build -p engine-context`

- [x] 3.1 РўРµСЃС‚ `builds_full_context`
  verify: `cargo test -p engine-context builds_full_context`

- [x] 3.2 РўРµСЃС‚ `includes_skip_protocol`
  verify: `cargo test -p engine-context includes_skip_protocol`

- [x] 3.3 РўРµСЃС‚ `truncates_oldest_turns`
  verify: `cargo test -p engine-context truncates_oldest_turns`

- [x] 3.4 РўРµСЃС‚ `keeps_short_dialogue`
  verify: `cargo test -p engine-context keeps_short_dialogue`

- [x] 3.5 РўРµСЃС‚ `appends_note`
  verify: `cargo test -p engine-context appends_note`

- [x] 4.1 `cargo clippy -p engine-context --all-targets -- -D warnings`
  verify: РІС‹С…РѕРґ 0

## STOP Protocol
Р•СЃР»Рё estimate_tokens РґР°С‘С‚ РЅРµРѕР¶РёРґР°РЅРЅС‹Рµ Р·РЅР°С‡РµРЅРёСЏ вЂ” РЅРµ РјРµРЅСЏС‚СЊ С„РѕСЂРјСѓР»Сѓ, РѕРЅР° Р·Р°С„РёРєСЃРёСЂРѕРІР°РЅР° СЃРїРµРєРѕР№.
РќРµ РґРѕР±Р°РІР»СЏС‚СЊ tiktoken Рё СЃРµС‚РµРІС‹Рµ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё. РћСЃС‚Р°РЅРѕРІРёС‚СЊСЃСЏ Рё СЃРїСЂРѕСЃРёС‚СЊ.

