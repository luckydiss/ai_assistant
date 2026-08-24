# Delta: VAD (state events)

## ADDED Requirements

### Requirement: Segmenter State Events
Segmenter SHALL эмитировать событие VadState {Waiting, Recording, Paused, Sending} при смене стадии.

#### Scenario: Порядок стадий (test: vad_state_sequence)
- GIVEN поток: тишина, речь 2с, тишина 700мс
- WHEN обработан
- THEN последовательность содержит Waiting→Recording→Paused→Sending

## MODIFIED / REMOVED: (none)
