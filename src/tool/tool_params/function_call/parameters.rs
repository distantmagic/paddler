use serde::Deserialize;
use serde::Serialize;

use super::parameters_schema::ParametersSchema;

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(untagged)]
pub enum Parameters {
    #[default]
    Empty,
    Schema(ParametersSchema),
}

impl Parameters {
    pub fn is_empty(&self) -> bool {
        matches!(self, Parameters::Empty)
    }
}
