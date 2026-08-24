# Tasks: TTS

- [ ] 1.1 TtsSection в config по design.md §1
  verify: `cargo test -p engine-config`

- [ ] 2.1 TtsClient в engine-llm по design.md §2
  verify: `cargo build -p engine-llm`

- [ ] 2.2 Тесты tts_returns_wav, tts_error_surfaced (mock-сервер, отдающий wav-байты)
  verify: `cargo test -p engine-llm tts`

- [ ] 3.1 player.rs по design.md §3
  verify: `cargo build -p desktop`

- [ ] 3.2 Тест playback_cancels_previous (unit: speak дважды, stream один)
  verify: `cargo test -p desktop playback_cancels_previous`

- [ ] 4.1 Wiring на Done + tts_toggle по design.md §4; тумблер в settings-view
  verify: manual — включённый tts озвучивает ответ

- [ ] 5.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0

## STOP Protocol
Если output-устройство не принимает f32-колбэк — добавить конвертацию под default_output_config().sample_format() по образцу capture.rs 003. Не менять протокол wav. Спросить при затруднении.
