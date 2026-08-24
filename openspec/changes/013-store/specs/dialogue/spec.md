# Delta: Dialogue (configurable thresholds)

## ADDED Requirements

### Requirement: Configurable Thresholds
Assembler SHALL поддерживать конструктор with_params(merge_ms, dedup_s, summary_threshold) для replay-тюнинга.

#### Scenario: Кастомные пороги (test: custom_merge_threshold)
- GIVEN Assembler::with_params(merge_ms=200, ...)
- WHEN две реплики с паузой 300мс
- THEN они НЕ склеены (в отличие от дефолта 500мс)

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
