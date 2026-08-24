# Tasks: IPC Wiring

- [x] 1.1 Р”РѕР±Р°РІРёС‚СЊ РІ workspace Cargo.toml Р·Р°РІРёСЃРёРјРѕСЃС‚Рё РёР· design.md В§1
  verify: `cargo metadata` РЅРµ РїР°РґР°РµС‚

- [x] 1.2 РћР±РЅРѕРІРёС‚СЊ `apps/desktop/Cargo.toml` РїРѕ design.md В§2
  verify: `cargo build -p desktop`

- [x] 2.1 Р—Р°РјРµРЅРёС‚СЊ `apps/desktop/src/main.rs` РЅР° design.md В§3
  verify: `cargo build -p desktop`

- [x] 2.2 РЎРѕР·РґР°С‚СЊ `apps/desktop/capabilities/default.json` РїРѕ design.md В§4
  verify: `cargo build -p desktop`

- [x] 3.1 Smoke: Р·Р°РїСѓСЃС‚РёС‚СЊ `cargo run -p desktop` СЃ config.toml Рё silero_vad.onnx
  verify: Р»РѕРіРё "System audio: ...Hz", РѕС€РёР±РѕРє РЅРµС‚ (manual)

- [x] 3.2 РџСЂРѕРІРµСЂРёС‚СЊ С…РѕС‚РєРµР№ Ctrl+Shift+Space (manual)
  verify: РІ Р»РѕРіР°С… РІРёРґРµРЅ Р·Р°РїСЂРѕСЃ Рє LLM

- [x] 3.3 РџСЂРѕРІРµСЂРёС‚СЊ Ctrl+Shift+H hide/show (manual)
  verify: РѕРєРЅРѕ СЃРєСЂС‹РІР°РµС‚СЃСЏ Рё РїРѕСЏРІР»СЏРµС‚СЃСЏ

## STOP Protocol
Р•СЃР»Рё tauri generate_context РїР°РґР°РµС‚ вЂ” РїСЂРѕРІРµСЂРёС‚СЊ РЅР°Р»РёС‡РёРµ frontendDist-РїР°РїРєРё (СЃРѕР·РґР°С‚СЊ Р·Р°РіР»СѓС€РєСѓ ui/index.html РёР· change 011).
Р•СЃР»Рё on_shortcut РЅРµ РєРѕРјРїРёР»РёСЂСѓРµС‚СЃСЏ вЂ” СЃРІРµСЂРёС‚СЊ РІРµСЂСЃРёСЋ РїР»Р°РіРёРЅР° "2" Рё СЃРёРЅС‚Р°РєСЃРёСЃ ShortcutState РёР· design.md В§3.
РќРµ РІС‹РЅРѕСЃРёС‚СЊ wiring РІ РѕС‚РґРµР»СЊРЅС‹Рµ РєСЂРµР№С‚С‹. РћСЃС‚Р°РЅРѕРІРёС‚СЊСЃСЏ Рё СЃРїСЂРѕСЃРёС‚СЊ.

