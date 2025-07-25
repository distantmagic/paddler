use std::path::PathBuf;

use crate::model_parameters::ModelParameters;

#[derive(Clone, Debug)]
pub struct AgentApplicableState {
    pub model_parameters: ModelParameters,
    pub model_path: PathBuf,
}
