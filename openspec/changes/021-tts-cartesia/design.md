# Design: Streaming TTS (Cartesia)

## 1. Workspace Cargo.toml

```toml
tokio-tungstenite = { version = "0.21", features = ["rustls-tls-webpki-roots"] }
# base64, uuid, futures, cpal, serde_json, tokio — уже в матрице
```

## 2. engine-tts: структура

```
crates/engine-tts/src/
├── lib.rs        // pub mod split; pub mod cartesia; pub mod player; pub mod feeder;
├── split.rs      // strip_code, split_sentences, resample_linear_f32
├── feeder.rs     // SentenceFeeder
├── cartesia.rs   // WS-сессия
├── player.rs     // F32Player
└── error.rs
```

## 3. split.rs

```rust
/// Вырезает ```-блоки; текст вне блоков проходит как есть.
pub fn strip_code(md: &str) -> String { ... }

/// (готовые предложения, остаток). Границы: . ! ? ; \n + пробелы после.
pub fn split_sentences(buf: &str, flush: bool) -> (Vec<String>, String) { ... }

pub fn resample_linear_f32(src: &[f32], from: u32, to: u32) -> Vec<f32> { ... }
```

## 4. feeder.rs

Инкрементальный SentenceFeeder: push_token → готовые предложения (код вырезан), finish → хвост.

## 5. cartesia.rs

WS wss://api.cartesia.ai/tts/websocket, заголовки X-API-Key + Cartesia-Version: 2024-06-10.
Сообщения: {"model_id","transcript","continue","voice":{mode,id},"output_format":{container:raw,encoding:pcm_f32le,sample_rate},"context_id"}.
Приём: Text {"type":"chunk","data":<base64 f32le>} / {"type":"done"} / {"type":"error"}; Binary → pcm.
`start_session` (продакшн URL) и `start_session_at(url)` (для mock-тестов).

## 6. player.rs

F32Player: очередь VecDeque<f32> + rate; push → линейный ресемпл + enqueue; pop/pop_front; clear.
Thread-based Player (cpal::Stream не Send на Windows): worker-поток владеет stream, Pull из общей очереди Arc<Mutex<VecDeque>>, ленивый старт устройства при первом push.

## 7. Config (engine-config)

TtsSection { mode, provider, api_key, model_id, voice_id, sample_rate } + HotkeysSection.tts.
Валидации: mode ∈ {off,auto,hotkey}; rate ∈ 8000..=44100; mode!=off && api_key пустой → Err(Validation).

## 8. Wiring (desktop)

AppServices += `tts: Mutex<TtsState>` (Player handle, session handle {cmd,abort}, feeder, last_answer, playing flag).
- Status("generating") → reset (abort+clear+feeder fresh) и, если auto, start_session + reader-таск (Pcm→player.push, Done/Error→playing=false).
- Token → last_answer += text; если сессия жива: feeder.push_token → cmd.send(Text).
- Done → feeder.finish + cmd.send(Flush).
- Error → reset.
- tts_play_last (команда + хоткей Ctrl+T + кнопка): если playing → стоп; иначе сессия + прогнать last_answer + Flush.
- stop_pipeline → reset.

## 9. Probe (examples/tts_probe.rs)

start_session с конфигом из config.toml; Text("Привет! Как дела?") + Flush; Pcm в probe.pcm; первый Pcm < 1000 мс.

## 10. Mock WS-сервер для тестов (tests/mock_ws.rs)

tokio TcpListener + accept_async; на continue:true → chunk (base64), на continue:false → done; записывает context_id.

## Рассмотрено и отклонено
- **Персистентное WS-соединение на всё приложение:** отклонено — сессия на ответ проще, хендшейк ~150 мс приемлем
- **44100:** отклонено по решению человека — 22050 достаточно и вдвое меньше данных
- **i16-формат:** отклонено — Cartesia f32le, cpal f32 нативно