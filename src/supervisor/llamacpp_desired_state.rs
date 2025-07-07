use serde::Deserialize;
use serde::Serialize;

use crate::supervisor::llamacpp_desired_model::LlamaCppDesiredModel;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LlamaCppDesiredState {
    pub model: LlamaCppDesiredModel,
}
