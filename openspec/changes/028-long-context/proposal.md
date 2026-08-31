# Proposal: Long-Context Memory

## Why
На боевом собеседовании 1 час (100+ реплик) текущий ContextBuilder усекает старейшие реплики и теряет ранний контекст: возврат интервьюера к теме из начала («а как вы решили задачу с градиентом?») уже не помнится. Нужна ограниченная по токенам память, сохраняющая суть всей сессии.

## What Changes
- Трёхслойная память в orchestrator (per chat): summary + key_turns + recent window
- Периодическая LLM-суммаризация: старейшие реплики сверх recent_window сжимаются в summary
- Key-turn detector: важные вопросы/ответы хранятся отдельно, не теряются
- ContextBuilder.build принимает структурированный ContextInput и компонует все слои

## Scope
- config [context]: recent_window/ key_turns_cap/ summary_max_tokens/ summary_model
- engine-orchestrator: ChatState += key_turns; drain+summarize; Cmd::SummaryDone
- engine-context: ContextInput + новая компоновка user-блока

## Non-Goals
- Векторный поиск по истории (RAG) — оверинжиниринг для MVP
- Суммаризация в реальном времени (только по границе окна)

## Affected Specs
- context (MODIFIED), orchestrator (ADDED), config (ADDED)
