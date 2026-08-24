# Report: Manual-Only

## Status: DONE

## Что сделано
- Удалена вся авто- и спекулятивная генерация: orchestrator больше не реагирует на реплики интервьюера запросом к LLM (только пополняет контекст), удалены debounce/min_words.
- Удалён SKIP-протокол целиком: `engine-llm/src/skip.rs`, `SkipState`, `AnswerEvent::Skipped`, `feed_skip`, `Phase` в client.rs, обработка `answer_skipped` в UI, `OrchEvent::Skipped`.
- `AnswerEvent` теперь: Token / Done / Failed.
- `LlmClient` шлёт `reasoning_effort` в тело запроса, если задан (через `with_reasoning_effort`).
- `ContextBuilder::build(summary, turns, focus: Option<&Turn>, note)`; user-блок заканчивается «Ответь по запросу кандидата.», без протокола.
- `config.rs`: удалена секция `[orchestrator]` (OrchestratorConfig), добавлено `llm.reasoning_effort: Option<String>`.
- `Orchestrator::new(ctx, llm)` — без debounce/min_words; `Cmd` = Turn / Manual; last-trigger-wins через abort предыдущего стрима.
- UI: подсказка в статусе listening «listening — Что сказать: Ctrl+Shift+Space».
- `config.toml`: temperature=0, reasoning_effort="low", system-промпт без SKIP.

## Отклонения от design.md
- **Хоткей toggle окна**: Ctrl+Shift+H занят сторонним приложением («HotKey already registered»). Заменён на Ctrl+Shift+O (Ctrl+Shift+Space для ручного запроса не затронут).
- **client.rs**: дополнительно `http1_only()` + `User-Agent: curl/8.0` + `Accept: text/event-stream` — стабилизирует ответы reasoning-модели deepseek-v4-flash-0731 (выявлено при диагностике).
- **config.toml** оставлен с реальными ключами (Groq STT, dslab LLM), без заглушек — приложение запускается сразу.

## Причина спеки (подтверждена на практике)
Модель deepseek-v4-flash-0731 недетерминирована: на идентичный запрос иногда возвращала `<SKIP>` (reasoning-траектория приводит к «это не вопрос»). При автоматическом триггере это ломало UX (ложные пропуски). Решение — генерировать только по явному запросу пользователя.

## Результат проверки (live, 18.08.2026)
- 3+ минуты речи интервьюера/кандидата → 0 спонтанных `generating` (только `listening`).
- 3 ручных запроса (Ctrl+Shift+Space) → 3 × `generating → orch done`, ни одного `skipped`.
- Ручной триггер работает даже без реплик интервьюера (focus=None → user-блок без «Последний вопрос I»).

## Тесты
- engine-config: 7 passed (убраны min_words/debounce, добавлен reasoning_effort=None default)
- engine-context: 7 passed (builds_without_focus, no_skip_instruction вместо includes_skip_protocol)
- engine-llm: 6 passed (reasoning_effort_sent; удалены skip-тесты)
- engine-orchestrator: 4 passed (manual_trigger_fires, manual_cancels_previous, turns_accumulate_no_fire, manual_includes_context)
- clippy --workspace --all-targets: чисто; cargo fmt: чисто.
- Примечание: `engine-audio` живые тесты (captures_system_audio и др.) флаки — ждут реальный системный звук 5с; на тихой машине падают, не связаны с этим change.