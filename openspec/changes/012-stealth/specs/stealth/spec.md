# Delta: Stealth

## ADDED Requirements

### Requirement: Capture Exclusion
Система SHALL исключать главное окно из захвата экрана через WDA_EXCLUDEFROMCAPTURE при старте.

#### Scenario: Affinity применён (test: affinity_applied)
- GIVEN запущенное приложение
- WHEN вызвана проверка GetWindowDisplayAffinity
- THEN возвращает флаг WDA_EXCLUDEFROMCAPTURE (automated example)

#### Scenario: Невидимость в шаринге (test: manual_zoom_share)
- GIVEN запущен Zoom с демонстрацией экрана
- WHEN оверлей с ответом открыт
- THEN зрители не видят оверлей (manual)

#### Scenario: Невидимость в OBS (test: manual_obs_capture)
- GIVEN OBS с Display Capture
- WHEN оверлей открыт
- THEN в OBS оверлей не виден (manual)

## MODIFIED Requirements
(none)

## REMOVED Requirements
(none)
