use serde::Deserialize;
use serde::Serialize;

use super::parameters::Parameters;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Function {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Parameters::is_empty")]
    pub parameters: Parameters,
}
