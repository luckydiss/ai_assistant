# Delta: Voice Activity Detection

## ADDED Requirements

### Requirement: VAD Model Loading
Система SHALL загружать Silero VAD ONNX модель из локального файла.

#### Scenario: Успешная загрузка модели (test: loads_model_successfully)
- GIVEN файл silero_vad.onnx существует
- WHEN вызван `VadProcessor::new("silero_vad.onnx")`
- THEN возвращается `Ok(VadProcessor)` с загруженной моделью
- AND модель готова к inference

#### Scenario: Файл модели отсутствует (test: errors_on_missing_model)
- GIVEN файл silero_vad.onnx не существует
- WHEN вызван `VadProcessor::new("silero_vad.onnx")`
- THEN возвращается `Err(VadError::ModelLoad)`

### Requirement: Speech Detection
Система SHALL определять наличие речи в аудио чанках используя Silero VAD.

#### Scenario: Речь в аудио (test: detects_speech)
- GIVEN VadProcessor и аудио с речью
- WHEN вызван `process_chunk(audio_chunk)`
- THEN возвращается `VadResult { speech: true, probability: >0.5 }`

#### Scenario: Тишина в аудио (test: detects_silence)
- GIVEN VadProcessor и аудио с тишиной
- WHEN вызван `process_chunk(audio_chunk)`
- THEN возвращается `VadResult { speech: false, probability: <0.5 }`

### Requirement: Segment Closing
Система SHALL закрывать речевой сегмент при паузе тишины ≥ 600мс или достижении 7 секунд.

#### Scenario: Пауза закрывает сегмент (test: closes_on_silence_600ms)
- GIVEN активный речевой сегмент и наступившая тишина
- WHEN тишина длится 600мс
- THEN сегмент закрывается
- AND emitted `SpeechSegment` с корректным audio data
- AND длительность сегмента ≤ 7000мс

#### Scenario: Длинная речь режется (test: splits_long_utterance)
- GIVEN непрерывная речь длительностью 15000мс
- WHEN обработан весь аудио поток
- THEN выдано ≥ 2 сегментов
- AND каждый сегмент ≤ 7000мс
- AND сегменты не перекрываются

### Requirement: Streaming Processing
Система SHALL поддерживать потоковую обработку аудио без накопления всего сегмента в памяти.

#### Scenario: Real-time processing (test: streams_audio)
- GIVEN аудио поток приходит чанками по 512 сэмплов
- WHEN каждый чанк обработан через `process_chunk`
- THEN memory usage не растет с временем
- AND сегменты emitted как только определены

### Requirement: Context Preservation
Система SHALL сохранять аудио контекст между чанками для корректной сегментации.

#### Scenario: Речь через паузу (test: preserves_context)
- GIVEN речь, пауза 200мс (< 600мс), речь продолжается
- WHEN обработан весь поток
- THEN emitted один сегмент (не два)
- AND сегмент содержит все аудио данные

## MODIFIED Requirements

(none)

## REMOVED Requirements

(none)
