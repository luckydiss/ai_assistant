# Delta: Context (languages flow)

## MODIFIED Requirements

### Requirement: Message Assembly
PromptContext SHALL содержать languages: Vec<String>; with_workspace сохраняет их; пайплайн передаёт их в orchestrator (set_languages) при старте встречи.

#### Scenario: Languages доходят (test: languages_propagate)
- GIVEN контекст languages=["ru","en"]
- WHEN start_pipeline
- THEN orchestrator знает 2 языка (assert через сеттер в unit-тесте)

## ADDED / REMOVED: (none)
