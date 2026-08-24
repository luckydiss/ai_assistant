# Design: Overlay v2

## 1. engine-vad: состояния

В `Segmenter` добавить `state_tx: broadcast::Sender<VadState>` и метод `subscribe_states()`. Эмитировать:
- Waiting — после emit_segment и при старте;
- Recording — первый речевой чанк нового сегмента;
- Paused — начался отсчёт тишины при непустом буфере;
- Sending — перед emit_segment.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VadState { Waiting, Recording, Paused, Sending }
```

## 2. engine-audio: mute

```rust
pub struct AudioEngine { ..., mic_muted: Arc<AtomicBool> }
pub fn set_mic_muted(&self, m: bool) { self.mic_muted.store(m, Ordering::SeqCst); }
// в mic-колбэке: if muted { return; } до отправки
```

## 3. pipeline.rs (вынос из main.rs)

```rust
pub struct PipelineHandle { stop_tx: Option<oneshot::Sender<()>> }
pub async fn start(app: tauri::AppHandle, store: Arc<SessionStore>, meeting_id: String) -> anyhow::Result<PipelineHandle>
pub async fn stop(h: &mut PipelineHandle)
```

Внутри start — весь wiring из main.rs change 010 + логирование 013 + `bump_messages` на turn/answer + эмит "vad" от subscribe_states. main.rs setup больше НЕ стартует пайплайн.

## 4. Окна

tauri.conf.json: окно main (1100x700, обычное, frontendDist ../ui, url index.html).
Overlay создаётся в setup:

```rust
let overlay = tauri::WebviewWindowBuilder::new(app, "overlay",
    tauri::WebviewUrl::App("overlay.html".into()))
    .transparent(true).decorations(false).always_on_top(true)
    .skip_taskbar(true).shadow(false).resizable(false)
    .inner_size(480.0, 640.0).position(900.0, 60.0).build()?;
stealth::apply_affinity(&overlay)?;
```

## 5. Новые команды (добавить в commands.rs)

```rust
#[tauri::command] async fn start_pipeline(app: AppHandle, state: State<'_, AppServices>, meeting_id: String)
#[tauri::command] async fn stop_pipeline(state: State<'_, AppServices>)
#[tauri::command] async fn mic_mute(state, muted: bool)
#[tauri::command] async fn protection_status(app: AppHandle) -> bool  // GetWindowDisplayAffinity == WDA_EXCLUDEFROMCAPTURE
#[tauri::command] async fn click_through(app: AppHandle, on: bool)    // overlay.set_ignore_cursor_events(on)
```

AppServices = { store, audio, orch, pipeline: Mutex<PipelineHandle> } — собрать в setup, app.manage().

## 6. overlay.html / overlay.js (статика, withGlobalTauri)

overlay.html: шапка (mute-кнопка, бейдж модели `#model`, бейдж защиты `#prot`), лента `#feed`, строка стадий VAD `#vad` (4 точки), quick-actions («Что сказать», «Резюме»), поле `#input` + send. CSS как в 011 + `.msg-i{color:#ffb454} .msg-c{color:#7ee787}`.

overlay.js (суть):

```js
const { event, core } = window.__TAURI__;
event.listen("turn", e => addMsg(e.payload.speaker === "Interviewer" ? "I" : "C", e.payload.text));
event.listen("answer_token", e => appendAnswer(e.payload));
event.listen("answer_done", () => finalizeAnswer());
event.listen("status", e => setBadge(e.payload));
event.listen("vad", e => setVadState(e.payload)); // Waiting/Recording/Paused/Sending
onMount: core.invoke("protection_status").then(ok => prot.textContent = ok ? "Защита вкл." : "Защита ОТКЛ.");
btnWhat.onclick = () => core.invoke("manual_trigger", { note: null });
btnResume.onclick = () => core.invoke("manual_trigger", { note: "сжато перескажи суть диалога" });
btnSend.onclick = () => core.invoke("manual_trigger", { note: input.value });
btnMute.onclick = () => core.invoke("mic_mute", { muted: toggle() });
```

## 7. index.html / app.js (main-окно)

Hash-роутер: `#meetings` (дефолт), `#contexts`.
- meetings view: `invoke("meetings_list")` → рендер групп по дате; форма создания (name+vacancy) → `meeting_create`; кнопка «Продолжить» → `meeting_create`-нет: `start_pipeline(id)` + показать overlay; поиск — фильтр на клиенте.
- contexts view: список `contexts_list`, форма полей ContextRow, файл-инпут читает TXT/MD через FileReader → resume_text; сохранение `context_save`; назначение `meeting_set_context`.

Код views — по образцу overlay.js: только invoke + рендер строк, без фреймворков.

## Рассмотрено и отклонено
- **SPA-фреймворк:** отклонено (нет npm)
- **Автозапуск пайплайна при старте приложения:** отклонено — только через «Продолжить»/создание
