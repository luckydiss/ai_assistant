# Report: Overlay v2

## Что сделано
- **engine-vad**: `VadState` (Waiting/Recording/Paused/Sending) + `Segmenter::subscribe_states()`, эмиссия состояний в `process_chunk` (set_state с дедупликацией).
- **engine-audio**: `mic_muted: Arc<AtomicBool>` + `set_mic_muted`, фильтрация в mic-колбэках (F32 и I16).
- **pipeline.rs**: вынесен весь wiring (audio→segmenters→stt→assembler→orch→store/logger/UI-эмиты), плюс эмит `vad`, `bump_messages` на turn/answer, stealth на overlay. `PipelineHandle` со stop через broadcast.
- **main.rs**: больше не стартует пайплайн. Собирает `AppServices`, управляет окнами (main + overlay), регистрирует hotkeys.
- **commands.rs**: 5 новых команд + manual_trigger перенесён в commands. Store/audio/orch/pipeline — через `State<Arc<AppServices>>`.
- **tauri.conf.json**: main 1100x700 обычное окно; overlay создаётся в setup (480x640, transparent, always-on-top, skip_taskbar, stealth).
- **UI**: overlay.html/overlay.js (лента, VAD-точки, бейджи model/prot, mute, quick-actions, input+send); index.html/app.js (hash-роутер meetings/contexts, формы, файл-импорт резюме, «Продолжить» → start_pipeline).
- Hotkeys: **Ctrl+2** — «Что сказать», **Ctrl+W** — click-through toggle, **Ctrl+Shift+H** — hide overlay.

## Отклонения от design.md
1. **`unsafe impl Send/Sync for AudioEngine`**: `cpal::Stream` помечен !Send/!Sync через `PhantomData<*mut ()>`, но на Windows WASAPI потоки потокобезопасны. Без этого `Arc<Mutex<AudioEngine>>` нельзя поместить в tauri State (требование AppServices по §5). Альтернативы (actor-поток/сырые указатели) сложнее.
2. **Хоткей «Что сказать» = Ctrl+2** вместо Ctrl+Shift+Space из предыдущих ченджей — по патч-листу manual-only (016: «Что сказать» = Ctrl+2). В design.md §5/§6 хоткей не фиксировался, кнопки quick-actions всё равно есть.
3. **Stop через `broadcast::Sender<()>`** вместо oneshot из design §3 — в пайплайне 5 задач, каждая подписывается; oneshot не клонируется.
4. **`protection_status`/`click_through`/`mic_mute` через `AppHandle`** — берут overlay/audio из app state; сигнатуры совместимы с design.
5. **VAD-эмит** — payload `{lane, state}` вместо голого VadState: два лейна (I/C) идут в один канал, нужна дискриминация.
6. **overlay.js не имеет обработчика answer_skipped** — по патч-листу (SKIP удалён).

## Результаты проверок
- `cargo build --workspace` — ok; `cargo clippy --workspace --all-targets -- -D warnings` — 0.
- Тесты: vad_state_sequence ok; mic_mute_stops_events ok; остальные крейты (store/context/dialogue/orchestrator/stt/config/llm) зелёные. Известные живые флаки engine-audio (captures_system_audio/dual_lane_capture/async_stream) — pre-existing, не связаны с ченджем.

## Осталось (manual)
- Боевая проверка: запуск, «Продолжить» стартует пайплайн, лента/стадии VAD/бейджи, mute, quick-actions, click-through (Ctrl+W), hide (Ctrl+Shift+H), защита overlay в захвате.