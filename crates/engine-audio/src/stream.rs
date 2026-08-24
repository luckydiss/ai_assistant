/// Байтовый чанк аудио (s16le, 16kHz, mono). Все чанки имеют чётную длину.
pub type AudioChunk = Vec<u8>;

pub type SystemAudioRx = tokio::sync::mpsc::UnboundedReceiver<AudioChunk>;
pub type MicAudioRx = tokio::sync::mpsc::UnboundedReceiver<AudioChunk>;