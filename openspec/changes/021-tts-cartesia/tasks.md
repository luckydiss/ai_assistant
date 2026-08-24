# Tasks: Streaming TTS

## Phase 1: Config
- [x] 1.1 TtsSection + хоткей tts + валидации по design.md §7
  verify: тесты tts_defaults, tts_requires_key, tts_validates_mode

## Phase 2: engine-tts ядро
- [x] 2.1 Создать crate, split.rs по design.md §3
  verify: тесты strip_code_removes_fences, split_by_sentence, flush_emits_tail, resample_length_ratio
- [x] 2.2 feeder.rs по design.md §4
  verify: тесты feeder_splits_sentences, feeder_skips_code, feeder_flush_tail

## Phase 3: Cartesia WS
- [x] 3.1 cartesia.rs по design.md §5
  verify: `cargo build -p engine-tts`
- [x] 3.2 mock_ws.rs по design.md §10
  verify: файл существует
- [x] 3.3 Тесты first_audio_before_done, finalize_sent, context_per_answer
  verify: `cargo test -p engine-tts`

## Phase 4: Плеер
- [x] 4.1 player.rs по design.md §6
  verify: тесты playback_order, player_clear

## Phase 5: Probe
- [ ] 5.1 examples/tts_probe.rs по design.md §9; прогнать с реальным ключом (человек)
  verify: первый Pcm < 1000 мс; `ffplay -f f32le -ar 22050 probe.pcm` слышен голос

## Phase 6: Wiring и UI
- [x] 6.1 TtsState + обработка OrchEvent + tts_play_last по design.md §8
  verify: `cargo build -p desktop`
- [x] 6.2 Хоткей tts (Ctrl+T) в dispatch + кнопка #btnSpeak в overlay.js
  verify: manual — кнопка и хоткей озвучивают/глушат
- [x] 6.3 Удалить старую озвучку 019 (TtsClient/wav/hound) по design.md §8
  verify: `cargo build --workspace` без ссылок на gemini-tts

## Phase 7: Боевая проверка
- [ ] 7.1 auto: звук начинается до конца генерации; новый ответ глушит предыдущую озвучку
- [ ] 7.2 hotkey: Ctrl+T озвучивает последний ответ, повторный — стоп; off: тишина
  verify: manual

## Phase 8: Валидация
- [x] 8.1 `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  verify: выход 0

## STOP Protocol
Если WS-хендшейк отклонён (401/403/upgrade error) — вывести сырой ответ сервера в лог и спросить человека; НЕ подбирать версии заголовков наугад.
Ключ в коде = немедленный стоп и вопрос. Не менять sample_rate с 22050.