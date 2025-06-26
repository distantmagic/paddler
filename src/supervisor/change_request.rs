use serde::Deserialize;
use serde::Serialize;

use crate::supervisor::llamacpp_state::LlamaCppState;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChangeRequest {
    pub desired_state: LlamaCppState,
}
