use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LlamaCppState {
    pub is_running: bool,
}
