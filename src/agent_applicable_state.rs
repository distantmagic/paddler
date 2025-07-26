use std::path::PathBuf;

use crate::inference_parameters::InferenceParameters;

#[derive(Clone, Debug)]
pub struct AgentApplicableState {
    pub inference_parameters: InferenceParameters,
    pub model_path: PathBuf,
}
