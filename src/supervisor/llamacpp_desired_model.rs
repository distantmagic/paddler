use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LlamaCppDesiredModel {
    HuggingFace(String),
    Local(String),
}
