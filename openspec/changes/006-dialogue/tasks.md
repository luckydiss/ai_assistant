# Tasks: Dialogue Assembler

## Phase 1: Dependencies

- [x] 1.1 РћР±РЅРѕРІРёС‚СЊ `crates/engine-dialogue/Cargo.toml` РґРѕР±Р°РІРёС‚СЊ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё РёР· design.md В§1
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

## Phase 2: Error Types

- [x] 2.1 РЎРѕР·РґР°С‚СЊ `crates/engine-dialogue/src/error.rs` РёР· design.md В§2
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

## Phase 3: Types

- [x] 3.1 РЎРѕР·РґР°С‚СЊ `crates/engine-dialogue/src/types.rs` СЃ Speaker, Transcript, Turn, Dialogue РёР· design.md В§3
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

## Phase 4: Assembler

- [x] 4.1 РЎРѕР·РґР°С‚СЊ `crates/engine-dialogue/src/assembler.rs` СЃ Assembler struct РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

- [x] 4.2 Р РµР°Р»РёР·РѕРІР°С‚СЊ `new()` РјРµС‚РѕРґ РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

- [x] 4.3 Р РµР°Р»РёР·РѕРІР°С‚СЊ `process_transcript()` РјРµС‚РѕРґ РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

- [x] 4.4 Р РµР°Р»РёР·РѕРІР°С‚СЊ `process_buffer()` helper РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

- [x] 4.5 Р РµР°Р»РёР·РѕРІР°С‚СЊ `is_garbage()` helper РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

- [x] 4.6 Р РµР°Р»РёР·РѕРІР°С‚СЊ `is_duplicate()` helper РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

- [x] 4.7 Р РµР°Р»РёР·РѕРІР°С‚СЊ `can_merge()` helper РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

- [x] 4.8 Р РµР°Р»РёР·РѕРІР°С‚СЊ `generate_summary()` helper РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

- [x] 4.9 Р РµР°Р»РёР·РѕРІР°С‚СЊ `get_dialogue()` РјРµС‚РѕРґ РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

- [x] 4.10 Р РµР°Р»РёР·РѕРІР°С‚СЊ `get_recent_turns()` РјРµС‚РѕРґ РёР· design.md В§4
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

## Phase 5: Public API

- [x] 5.1 РћР±РЅРѕРІРёС‚СЊ `crates/engine-dialogue/src/lib.rs` СЃ pub use РёР· design.md В§1
  verify: `cargo build -p engine-dialogue` РїСЂРѕС…РѕРґРёС‚

## Phase 6: Tests

- [x] 6.1 РЎРѕР·РґР°С‚СЊ `crates/engine-dialogue/tests/dialogue_tests.rs`
  verify: С„Р°Р№Р» СЃРѕР·РґР°РЅ

- [x] 6.2 РўРµСЃС‚ `orders_by_timestamp` (scenario РёР· specs) - out-of-order transcripts
  verify: `cargo test -p engine-dialogue orders_by_timestamp` РїСЂРѕС…РѕРґРёС‚

- [x] 6.3 РўРµСЃС‚ `handles_same_timestamp` (scenario РёР· specs) - same start time
  verify: `cargo test -p engine-dialogue handles_same_timestamp` РїСЂРѕС…РѕРґРёС‚

- [x] 6.4 РўРµСЃС‚ `merges_short_pause` (scenario РёР· specs) - 200ms pause
  verify: `cargo test -p engine-dialogue merges_short_pause` РїСЂРѕС…РѕРґРёС‚

- [x] 6.5 РўРµСЃС‚ `splits_long_pause` (scenario РёР· specs) - 800ms pause
  verify: `cargo test -p engine-dialogue splits_long_pause` РїСЂРѕС…РѕРґРёС‚

- [x] 6.6 РўРµСЃС‚ `filters_exact_duplicate` (scenario РёР· specs) - same text within 2s
  verify: `cargo test -p engine-dialogue filters_exact_duplicate` РїСЂРѕС…РѕРґРёС‚

- [x] 6.7 РўРµСЃС‚ `keeps_similar_text` (scenario РёР· specs) - "Hello" vs "Hello!"
  verify: `cargo test -p engine-dialogue keeps_similar_text` РїСЂРѕС…РѕРґРёС‚

- [x] 6.8 РўРµСЃС‚ `filters_short_utterance` (scenario РёР· specs) - 1 word
  verify: `cargo test -p engine-dialogue filters_short_utterance` РїСЂРѕС…РѕРґРёС‚

- [x] 6.9 РўРµСЃС‚ `filters_filler_word` (scenario РёР· specs) - "РѕРє", "Р°РіР°"
  verify: `cargo test -p engine-dialogue filters_filler_word` РїСЂРѕС…РѕРґРёС‚

- [x] 6.10 РўРµСЃС‚ `keeps_valid_short_reply` (scenario РёР· specs) - "Р”Р°" as answer
  verify: `cargo test -p engine-dialogue keeps_valid_short_reply` РїСЂРѕС…РѕРґРёС‚

- [x] 6.11 РўРµСЃС‚ `generates_summary` (scenario РёР· specs) - 20 turns
  verify: `cargo test -p engine-dialogue generates_summary` РїСЂРѕС…РѕРґРёС‚

- [x] 6.12 РўРµСЃС‚ `updates_summary` (scenario РёР· specs) - existing summary + new turns
  verify: `cargo test -p engine-dialogue updates_summary` РїСЂРѕС…РѕРґРёС‚

## Phase 7: Integration Test

- [x] 7.1 РЎРѕР·РґР°С‚СЊ `examples/dialogue_demo.rs` РёР· design.md В§5
  verify: `cargo run -p engine-dialogue --example dialogue_demo` Р·Р°РїСѓСЃРєР°РµС‚СЃСЏ

- [x] 7.2 Р—Р°РїСѓСЃС‚РёС‚СЊ example Рё РїСЂРѕРІРµСЂРёС‚СЊ РєРѕСЂСЂРµРєС‚РЅРѕСЃС‚СЊ РґРёР°Р»РѕРіР° (manual)
  verify: turns СѓРїРѕСЂСЏРґРѕС‡РµРЅС‹ РїРѕ РІСЂРµРјРµРЅРё, garbage РѕС‚С„РёР»СЊС‚СЂРѕРІР°РЅ

## Phase 8: Validation

- [x] 8.1 Р—Р°РїСѓСЃС‚РёС‚СЊ `cargo clippy -p engine-dialogue --all-targets -- -D warnings`
  verify: РІС‹С…РѕРґ 0

- [x] 8.2 Р—Р°РїСѓСЃС‚РёС‚СЊ `cargo test -p engine-dialogue`
  verify: РІСЃРµ С‚РµСЃС‚С‹ РїСЂРѕС…РѕРґСЏС‚

- [x] 8.3 Р—Р°РїСѓСЃС‚РёС‚СЊ `cargo build -p engine-dialogue --release`
  verify: РІС‹С…РѕРґ 0

## STOP Protocol

Р•СЃР»Рё:
- BinaryHeap РЅРµ СЃРѕСЂС‚РёСЂСѓРµС‚ РїРѕ timestamp в†’ РїСЂРѕРІРµСЂРёС‚СЊ С‡С‚Рѕ Reverse РёСЃРїРѕР»СЊР·СѓРµС‚СЃСЏ РєРѕСЂСЂРµРєС‚РЅРѕ
- Merge Р»РѕРіРёРєР° РѕР±СЉРµРґРёРЅСЏРµС‚ СЃР»РёС€РєРѕРј РјРЅРѕРіРѕ в†’ РїСЂРѕРІРµСЂРёС‚СЊ can_merge СЃ tracing
- Summary РіРµРЅРµСЂРёСЂСѓРµС‚СЃСЏ СЃР»РёС€РєРѕРј С‡Р°СЃС‚Рѕ в†’ СѓРІРµР»РёС‡РёС‚СЊ summary_threshold

РќРµ РїС‹С‚Р°С‚СЊСЃСЏ РґРѕР±Р°РІРёС‚СЊ LLM-based summarization РёР»Рё complex NLP. РћСЃС‚Р°РЅРѕРІРёС‚СЊСЃСЏ Рё СЃРїСЂРѕСЃРёС‚СЊ.

