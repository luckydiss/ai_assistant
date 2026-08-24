# Delta: Config (ui/window/chat/stt/llm additions)

## ADDED Requirements

### Requirement: UI Section
[ui]: accent (default "#f97316"), opacity (default 92, 10..=100), indicator_corner (default "top-right", ∈ 4 углов).

### Requirement: Window Section
[window]: no_focus (default false), move_step (default 50), resize_step (default 50).

### Requirement: Chat Section
[chat]: order ("bottom"|"top", default bottom), font_size (default 13.5), code_theme (default "github-dark"), code_scroll (default true), autoscroll (default true), autoscroll_speed (default 100), collapse_transcripts (true), collapse_operations (true), collapse_last (false), compact_quick (true), cancel_on_resend (true), cancel_mode ("drop"|"keep", default drop).

### Requirement: STT Language
[stt] language (default "auto", ∈ auto|ru|en).

### Requirement: LLM Search
[llm] search_enabled (default false), search_tool_json (default "").

#### Scenario: Валидация opacity (test: validates_opacity)
- GIVEN opacity=150 → Err(Validation)

## MODIFIED / REMOVED: (none)
