use rubato::{FftFixedInOut, Resampler};

pub struct AudioResampler {
    resampler: FftFixedInOut<f32>,
    input_buffer: Vec<Vec<f32>>,
    output_buffer: Vec<Vec<f32>>,
    pending: Vec<f32>,
}

impl AudioResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Result<Self, crate::AudioError> {
        if input_rate == output_rate {
            return Err(crate::AudioError::UnsupportedSampleRate(input_rate));
        }

        let resampler =
            FftFixedInOut::<f32>::new(input_rate as usize, output_rate as usize, 1024, 1)
                .map_err(|e| crate::AudioError::Resampler(e.to_string()))?;

        let input_buffer = resampler.input_buffer_allocate(true);
        let output_buffer = resampler.output_buffer_allocate(true);

        Ok(Self {
            resampler,
            input_buffer,
            output_buffer,
            pending: Vec::new(),
        })
    }

    pub fn needed_input_frames(&self) -> usize {
        self.resampler.input_frames_next()
    }

    pub fn process(&mut self, input: &[f32]) -> Result<Vec<f32>, crate::AudioError> {
        let needed = self.resampler.input_frames_next();
        if needed == 0 {
            return Ok(input.to_vec());
        }

        self.pending.extend_from_slice(input);

        let mut output = Vec::new();
        while self.pending.len() >= needed {
            for (i, sample) in self.pending.drain(..needed).enumerate() {
                self.input_buffer[0][i] = sample;
            }

            let (frames_in, frames_out) = self
                .resampler
                .process_into_buffer(&self.input_buffer, &mut self.output_buffer, None)
                .map_err(|e| crate::AudioError::Resampler(e.to_string()))?;

            if frames_in == 0 {
                break;
            }

            output.extend_from_slice(&self.output_buffer[0][..frames_out]);
        }

        Ok(output)
    }
}
