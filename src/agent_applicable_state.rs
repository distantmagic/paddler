use std::path::PathBuf;

use crate::inference_parameters::InferenceParameters;
use crate::chat_template::ChatTemplate;

#[derive(Clone, Debug)]
pub struct AgentApplicableState {
    pub inference_parameters: InferenceParameters,
    pub model_path: Option<PathBuf>,
    pub override_chat_template: Option<ChatTemplate>,
}
