use crate::AudioError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AudioEngine {
    streams: Vec<Stream>,
    mic_muted: Arc<AtomicBool>,
}

// cpal::Stream помечен !Send/!Sync через PhantomData<*mut ()>, но на Windows (WASAPI)
// потоки реально потокобезопасны: их можно создавать, останавливать и дропать из
// любого потока. AppServices (design 016) требует AudioEngine: Send+Sync для tauri State.
unsafe impl Send for AudioEngine {}
unsafe impl Sync for AudioEngine {}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
            mic_muted: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Останавливает все потоки захвата (drop cpal::Stream останавливает WASAPI).
    pub fn stop(&mut self) {
        let count = self.streams.len();
        self.streams.clear();
        if count > 0 {
            tracing::info!("audio streams stopped: {count}");
        }
    }

    /// Количество активных потоков захвата.
    pub fn active_streams(&self) -> usize {
        self.streams.len()
    }

    pub fn set_mic_muted(&self, muted: bool) {
        self.mic_muted.store(muted, Ordering::SeqCst);
        tracing::info!("mic mute: {muted}");
    }

    pub fn mic_muted(&self) -> bool {
        self.mic_muted.load(Ordering::SeqCst)
    }

    /// Захват системного аудио (WASAPI loopback). Возвращает очередь s16le-байт @16kHz mono.
    pub fn start_system_capture(&mut self) -> Result<mpsc::UnboundedReceiver<Vec<u8>>, AudioError> {
        let (sender, receiver) = mpsc::unbounded_channel::<Vec<u8>>();

        let host = cpal::host_from_id(cpal::HostId::Wasapi)?;
        let device = host.default_output_device().ok_or(AudioError::NoDevice)?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        tracing::info!("System audio: {}Hz, {} channels", sample_rate, channels);

        let mut resampler = if sample_rate != 16000 {
            Some(crate::resampler::AudioResampler::new(sample_rate, 16000)?)
        } else {
            None
        };

        let stream_config = StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono = Self::to_mono(data, channels);
                    let resampled = Self::resample(&mut resampler, &mono, "system");
                    if !resampled.is_empty() {
                        let _ = sender.send(Self::to_s16le(&resampled));
                    }
                },
                |err| tracing::error!("System audio error: {}", err),
                None,
            )?,
            SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    let mono = Self::to_mono(&f32_data, channels);
                    let resampled = Self::resample(&mut resampler, &mono, "system");
                    if !resampled.is_empty() {
                        let _ = sender.send(Self::to_s16le(&resampled));
                    }
                },
                |err| tracing::error!("System audio error: {}", err),
                None,
            )?,
            _ => return Err(AudioError::UnsupportedSampleRate(sample_rate)),
        };

        stream.play()?;
        self.streams.push(stream);

        Ok(receiver)
    }

    /// Захват микрофона. Возвращает очередь s16le-байт @16kHz mono.
    pub fn start_mic_capture(
        &mut self,
        device_name: Option<&str>,
    ) -> Result<mpsc::UnboundedReceiver<Vec<u8>>, AudioError> {
        let (sender, receiver) = mpsc::unbounded_channel::<Vec<u8>>();

        let host = cpal::default_host();
        let device = match device_name {
            Some(name) => host
                .input_devices()?
                .find(|d| d.name().ok().as_deref() == Some(name))
                .ok_or(AudioError::NoDevice)?,
            None => host.default_input_device().ok_or(AudioError::NoDevice)?,
        };

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        tracing::info!("Mic audio: {}Hz, {} channels", sample_rate, channels);

        let muted = self.mic_muted.clone();
        let mut resampler = if sample_rate != 16000 {
            Some(crate::resampler::AudioResampler::new(sample_rate, 16000)?)
        } else {
            None
        };

        let stream_config = StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = match config.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if muted.load(Ordering::SeqCst) {
                        return;
                    }
                    let mono = Self::to_mono(data, channels);
                    let resampled = Self::resample(&mut resampler, &mono, "mic");
                    if !resampled.is_empty() {
                        let _ = sender.send(Self::to_s16le(&resampled));
                    }
                },
                |err| tracing::error!("Mic audio error: {}", err),
                None,
            )?,
            SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    if muted.load(Ordering::SeqCst) {
                        return;
                    }
                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    let mono = Self::to_mono(&f32_data, channels);
                    let resampled = Self::resample(&mut resampler, &mono, "mic");
                    if !resampled.is_empty() {
                        let _ = sender.send(Self::to_s16le(&resampled));
                    }
                },
                |err| tracing::error!("Mic audio error: {}", err),
                None,
            )?,
            _ => return Err(AudioError::UnsupportedSampleRate(sample_rate)),
        };

        stream.play()?;
        self.streams.push(stream);

        Ok(receiver)
    }

    fn resample(
        resampler: &mut Option<crate::resampler::AudioResampler>,
        mono: &[f32],
        label: &str,
    ) -> Vec<f32> {
        match resampler {
            Some(ref mut r) => match r.process(mono) {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!("{label} resample error: {e}");
                    Vec::new()
                }
            },
            None => mono.to_vec(),
        }
    }

    fn to_mono(data: &[f32], channels: u16) -> Vec<f32> {
        if channels == 1 {
            return data.to_vec();
        }

        let mut mono = Vec::with_capacity(data.len() / channels as usize);
        for chunk in data.chunks(channels as usize) {
            let sum: f32 = chunk.iter().sum();
            mono.push(sum / channels as f32);
        }
        mono
    }

    fn to_s16le(samples: &[f32]) -> Vec<u8> {
        let mut out = Vec::with_capacity(samples.len() * 2);
        for &s in samples {
            let v = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
            out.extend_from_slice(&v.to_le_bytes());
        }
        out
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop();
    }
}