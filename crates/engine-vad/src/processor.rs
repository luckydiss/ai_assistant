use crate::{VadError, VadResult};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;

const SAMPLE_RATE: i64 = 16000;
const CHUNK_SAMPLES: usize = 512;
const CONTEXT_SAMPLES: usize = 64;
const INPUT_SAMPLES: usize = CHUNK_SAMPLES + CONTEXT_SAMPLES;

pub struct VadProcessor {
    session: Session,
    state: Vec<f32>,
    context: Vec<f32>,
}

impl VadProcessor {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self, VadError> {
        let session = Session::builder()
            .map_err(|e| VadError::ModelLoad(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| VadError::ModelLoad(e.to_string()))?
            .with_intra_threads(1)
            .map_err(|e| VadError::ModelLoad(e.to_string()))?
            .with_inter_threads(1)
            .map_err(|e| VadError::ModelLoad(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| VadError::ModelLoad(e.to_string()))?;

        let state = vec![0.0f32; 256];
        let context = vec![0.0f32; CONTEXT_SAMPLES];

        Ok(Self {
            session,
            state,
            context,
        })
    }

    pub fn process_chunk(&mut self, audio: &[f32]) -> Result<VadResult, VadError> {
        if audio.is_empty() {
            return Ok(VadResult {
                speech: false,
                probability: 0.0,
            });
        }

        if audio.len() > CHUNK_SAMPLES {
            return Err(VadError::InvalidFormat);
        }

        let mut input = vec![0.0f32; INPUT_SAMPLES];
        input[..CONTEXT_SAMPLES].copy_from_slice(&self.context);
        input[CONTEXT_SAMPLES..CONTEXT_SAMPLES + audio.len()].copy_from_slice(audio);

        let input_tensor = Tensor::from_array(([1usize, INPUT_SAMPLES], input.clone()))
            .map_err(|e| VadError::Inference(e.to_string()))?;

        let state_tensor = Tensor::from_array(([2usize, 1, 128], self.state.clone()))
            .map_err(|e| VadError::Inference(e.to_string()))?;

        let sr_tensor = Tensor::from_array(([1usize], vec![SAMPLE_RATE]))
            .map_err(|e| VadError::Inference(e.to_string()))?;

        let outputs = self.session.run(ort::inputs![
            "input" => input_tensor,
            "state" => state_tensor,
            "sr" => sr_tensor
        ])?;

        let output = outputs
            .get("output")
            .ok_or_else(|| VadError::Inference("Missing 'output'".to_string()))?;
        let (_, output_data) = output
            .try_extract_tensor::<f32>()
            .map_err(|e| VadError::Inference(e.to_string()))?;
        let probability = output_data[0];

        if let Some(state_out) = outputs.get("stateN").or_else(|| outputs.get("state")) {
            let (_, state_data) = state_out
                .try_extract_tensor::<f32>()
                .map_err(|e| VadError::Inference(e.to_string()))?;
            self.state = state_data.to_vec();
        }

        let last_chunk = if audio.len() >= CHUNK_SAMPLES {
            &audio[audio.len() - CONTEXT_SAMPLES..]
        } else {
            audio
        };
        self.context.clear();
        self.context.extend_from_slice(last_chunk);
        self.context.resize(CONTEXT_SAMPLES, 0.0);

        Ok(VadResult {
            speech: probability > 0.5,
            probability,
        })
    }

    pub fn reset(&mut self) {
        self.state.fill(0.0);
        self.context.fill(0.0);
    }
}
