# Tasks: Orchestrator

- [x] 1.1 РћР±РЅРѕРІРёС‚СЊ `crates/engine-orchestrator/Cargo.toml` РїРѕ design.md В§1
  verify: `cargo build -p engine-orchestrator`

- [x] 2.1 РЎРѕР·РґР°С‚СЊ `src/orchestrator.rs` РїРѕ design.md В§3
  verify: `cargo build -p engine-orchestrator`

- [x] 2.2 РћР±РЅРѕРІРёС‚СЊ `src/lib.rs` РїРѕ design.md В§2
  verify: `cargo build -p engine-orchestrator`

- [x] 3.0 РЎРєРѕРїРёСЂРѕРІР°С‚СЊ `tests/mock.rs` РёР· engine-llm
  verify: С„Р°Р№Р» СЃСѓС‰РµСЃС‚РІСѓРµС‚

- [x] 3.1 РўРµСЃС‚ `triggers_after_debounce` (С€Р°Р±Р»РѕРЅ РІ design.md В§4)
  verify: `cargo test -p engine-orchestrator triggers_after_debounce`

- [x] 3.2 РўРµСЃС‚ `short_turn_ignored`
  verify: `cargo test -p engine-orchestrator short_turn_ignored`

- [x] 3.3 РўРµСЃС‚ `speculative_trigger_fast`
  verify: `cargo test -p engine-orchestrator speculative_trigger_fast`

- [x] 3.4 РўРµСЃС‚ `new_trigger_cancels_previous`
  verify: `cargo test -p engine-orchestrator new_trigger_cancels_previous`

- [x] 3.5 РўРµСЃС‚ `skip_hidden_from_ui`
  verify: `cargo test -p engine-orchestrator skip_hidden_from_ui`

- [x] 3.6 РўРµСЃС‚ `manual_trigger_fires`
  verify: `cargo test -p engine-orchestrator manual_trigger_fires`

- [x] 4.1 `cargo clippy -p engine-orchestrator --all-targets -- -D warnings`
  verify: РІС‹С…РѕРґ 0

## STOP Protocol
Р•СЃР»Рё deadlock РІ driver-С†РёРєР»Рµ вЂ” РќР• РґРѕР±Р°РІР»СЏС‚СЊ lock РІРЅСѓС‚СЂРё spawn-С„РѕСЂРІР°СЂРґРµСЂР°; СЃРІРµСЂРёС‚СЊСЃСЏ СЃ design.md В§3.
РќРµ РґРѕР±Р°РІР»СЏС‚СЊ РЅРѕРІС‹Рµ СЃРѕСЃС‚РѕСЏРЅРёСЏ РјР°С€РёРЅС‹ Р±РµР· РёР·РјРµРЅРµРЅРёСЏ СЃРїРµРєРё. РћСЃС‚Р°РЅРѕРІРёС‚СЊСЃСЏ Рё СЃРїСЂРѕСЃРёС‚СЊ.

