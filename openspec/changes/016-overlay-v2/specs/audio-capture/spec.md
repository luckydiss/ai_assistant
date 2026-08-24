# Delta: Audio (mic mute)

## ADDED Requirements

### Requirement: Mic Mute Flag
AudioEngine SHALL поддерживать set_mic_muted(bool); в muted MicData-события не эмитируются.

#### Scenario: Mute отключает lane C (test: mic_mute_stops_events)
- GIVEN mute включён
- WHEN приходит звук с микрофона
- THEN MicData не эмитируется, SystemData продолжают идти

## MODIFIED / REMOVED: (none)
