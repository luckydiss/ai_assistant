use crate::split::resample_linear_f32;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex};

pub struct F32Player {
    queue: Arc<Mutex<VecDeque<f32>>>,
    rate: u32,
}

impl F32Player {
    pub fn new(rate: u32) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            rate,
        }
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn set_rate(&mut self, rate: u32) {
        self.rate = rate;
    }

    pub fn push(&mut self, samples: &[f32], from_rate: u32) {
        let r = resample_linear_f32(samples, from_rate, self.rate);
        self.queue.lock().unwrap().extend(r);
    }

    pub fn pop(&self) -> f32 {
        self.queue.lock().unwrap().pop_front().unwrap_or(0.0)
    }

    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.queue.lock().unwrap().clear();
    }

    fn queue_arc(&self) -> Arc<Mutex<VecDeque<f32>>> {
        self.queue.clone()
    }
}

enum PlayCmd {
    Push(Vec<f32>, u32),
    Clear,
}

#[derive(Clone)]
pub struct Player {
    tx: mpsc::Sender<PlayCmd>,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<PlayCmd>();
        let _ = std::thread::Builder::new()
            .name("tts-playback".into())
            .spawn(move || run(rx));
        Self { tx }
    }

    pub fn push(&self, samples: Vec<f32>, from_rate: u32) -> anyhow::Result<()> {
        self.tx
            .send(PlayCmd::Push(samples, from_rate))
            .map_err(|_| anyhow::anyhow!("tts player thread gone"))
    }

    pub fn clear(&self) {
        let _ = self.tx.send(PlayCmd::Clear);
    }
}

fn run(rx: mpsc::Receiver<PlayCmd>) {
    let mut sink = F32Player::new(22050);
    let mut stream: Option<cpal::Stream> = None;
    while let Ok(cmd) = rx.recv() {
        match cmd {
            PlayCmd::Clear => {
                stream = None;
                sink.clear();
            }
            PlayCmd::Push(samples, from) => {
                if stream.is_none() {
                    match start_output(&mut sink) {
                        Ok(s) => stream = Some(s),
                        Err(e) => {
                            tracing::error!("tts playback: {e}");
                            continue;
                        }
                    }
                }
                sink.push(&samples, from);
            }
        }
    }
}

fn start_output(sink: &mut F32Player) -> anyhow::Result<cpal::Stream> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("no output device"))?;
    let fmt = device.default_output_config()?;
    let rate = fmt.sample_rate().0;
    let channels = fmt.channels();
    sink.set_rate(rate);
    let q = sink.queue_arc();
    let scfg = cpal::StreamConfig {
        channels,
        sample_rate: fmt.sample_rate(),
        buffer_size: cpal::BufferSize::Default,
    };
    let stream = match fmt.sample_format() {
        cpal::SampleFormat::F32 => {
            let q = q.clone();
            device.build_output_stream(
                &scfg,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut q = q.lock().unwrap();
                    for frame in data.chunks_mut(channels as usize) {
                        let s = q.pop_front().unwrap_or(0.0);
                        for c in frame.iter_mut() {
                            *c = s;
                        }
                    }
                },
                |e| tracing::error!("playback: {e}"),
                None,
            )?
        }
        cpal::SampleFormat::I16 => {
            let q = q.clone();
            device.build_output_stream(
                &scfg,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let mut q = q.lock().unwrap();
                    for frame in data.chunks_mut(channels as usize) {
                        let s = q.pop_front().unwrap_or(0.0);
                        for c in frame.iter_mut() {
                            *c = (s * 32768.0) as i16;
                        }
                    }
                },
                |e| tracing::error!("playback: {e}"),
                None,
            )?
        }
        cpal::SampleFormat::U16 => {
            let q = q.clone();
            device.build_output_stream(
                &scfg,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    let mut q = q.lock().unwrap();
                    for frame in data.chunks_mut(channels as usize) {
                        let v = (q.pop_front().unwrap_or(0.0) + 1.0) * 32767.5;
                        let s = v.clamp(0.0, 65535.0) as u16;
                        for c in frame.iter_mut() {
                            *c = s;
                        }
                    }
                },
                |e| tracing::error!("playback: {e}"),
                None,
            )?
        }
        other => return Err(anyhow::anyhow!("unsupported output sample format: {other:?}")),
    };
    stream.play()?;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_order() {
        let mut p = F32Player::new(22050);
        let a: Vec<f32> = vec![1.0, 2.0, 3.0];
        let b: Vec<f32> = vec![4.0, 5.0];
        p.push(&a, 22050);
        p.push(&b, 22050);
        let mut got = Vec::new();
        while !p.is_empty() {
            got.push(p.pop());
        }
        assert_eq!(got, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn player_clear() {
        let mut p = F32Player::new(22050);
        p.push(&[1.0, 2.0], 22050);
        assert_eq!(p.len(), 2);
        p.clear();
        assert!(p.is_empty());
        assert_eq!(p.pop(), 0.0);
    }
}