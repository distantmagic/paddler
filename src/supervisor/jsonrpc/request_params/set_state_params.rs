use serde::Deserialize;
use serde::Serialize;

use super::RequestParams;
use crate::supervisor::llamacpp_state::LlamaCppState;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SetStateParams {
    pub desired_state: LlamaCppState,
    pub request_id: String,
}

impl RequestParams for SetStateParams {
    fn request_id(&self) -> String {
        self.request_id.clone()
    }
}
