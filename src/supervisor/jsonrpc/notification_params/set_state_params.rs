use serde::Deserialize;
use serde::Serialize;

use crate::llamacpp::llamacpp_state::LlamaCppState;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SetStateParams {
    pub desired_state: LlamaCppState,
}
