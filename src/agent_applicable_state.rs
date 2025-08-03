use std::path::PathBuf;

use crate::inference_parameters::InferenceParameters;
use crate::chat_template::ChatTemplate;

#[derive(Clone, Debug)]
pub struct AgentApplicableState {
    pub chat_template_override: Option<ChatTemplate>,
    pub inference_parameters: InferenceParameters,
    pub model_path: Option<PathBuf>,
}
