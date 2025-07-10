use serde::Deserialize;
use serde::Serialize;

use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SetStateParams {
    pub desired_state: LlamaCppDesiredState,
}
