# Design: TTS

## 1. Config

```toml
[tts]
enabled = false
model = "tts-1"
voice = "alloy"
```
```rust
#[serde(default)] pub tts: TtsSection  // enabled: bool, model: String, voice: String + Default
```

## 2. TtsClient (engine-llm/src/tts.rs)

```rust
pub struct TtsClient { http: Client, base_url: String, api_key: String, model: String, voice: String }

impl TtsClient {
    pub async fn synth_wav(&self, text: &str) -> Result<Vec<u8>, SttError> {
        let body = serde_json::json!({ "model": self.model, "voice": self.voice,
            "input": text, "response_format": "wav" });
        let resp = self.http.post(format!("{}/audio/speech", self.base_url))
            .bearer_auth(&self.api_key).json(&body).send().await?;
        if !resp.status().is_success() {
            return Err(SttError::Api { status: resp.status().as_u16(), message: "tts".into() });
        }
        Ok(resp.bytes().await?.to_vec())
    }
}
```

## 3. Player (apps/desktop/src/player.rs)

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::io::Cursor;

pub struct Player { stream: Option<cpal::Stream>, queue: Arc<Mutex<VecDeque<f32>>> }

impl Player {
    pub fn speak(&mut self, wav: &[u8]) -> anyhow::Result<()> {
        self.stop();
        let reader = hound::WavReader::new(Cursor::new(wav.to_vec()))?;
        let spec = reader.spec();
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => reader.into_samples::<i16>()
                .filter_map(|s| s.ok()).map(|s| s as f32 / 32768.0).collect(),
            hound::SampleFormat::Float => reader.into_samples::<f32>()
                .filter_map(|s| s.ok()).collect(),
        };
        let queue = Arc::new(Mutex::new(VecDeque::from(samples)));
        let host = cpal::default_host();
        let device = host.default_output_device().anyhow::Ok_or_else(|| anyhow::anyhow!("no output"))?;
        let cfg = cpal::StreamConfig {
            channels: spec.channels,
            sample_rate: cpal::SampleRate(spec.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };
        let q = queue.clone();
        let stream = device.build_output_stream(&cfg,
            move |data: &mut [f32], _| {
                let mut q = q.lock().unwrap();
                for s in data.iter_mut() { *s = q.pop_front().unwrap_or(0.0); }
            },
            |e| tracing::error!("playback: {e}"), None)?;
        stream.play()?;
        self.stream = Some(stream);
        self.queue = queue;
        Ok(())
    }
    pub fn stop(&mut self) { self.stream.take(); } // drop = остановка
}
```

Примечание: build_output_stream с f32-колбэком валиден для устройств с F32 format (стандарт WASAPI). Если BuildStreamError — ловить и логировать, не падать.

## 4. Wiring

В форвардере OrchEvent в main.rs: на `Done(full_text)` если tts.enabled → spawn: tts.synth_wav(full) → player.speak(wav). Player в AppServices (Mutex<Player>). Команда `tts_toggle` пишет в config (save) — hot-reload подхватит.

## Рассмотрено и отклонено
- **symphonia/mp3:** отклонено — просим wav у endpoint
- **Стриминг TTS:** отклонено
