use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ChatTemplateDoesNotCompileParams {
    pub error: String,
    pub template_content: String,
}
