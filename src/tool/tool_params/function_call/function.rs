use serde::Deserialize;
use serde::Serialize;

use super::parameters::Parameters;

#[derive(Debug, Deserialize, Serialize)]
pub struct Function {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Parameters::is_empty")]
    pub parameters: Parameters,
}
