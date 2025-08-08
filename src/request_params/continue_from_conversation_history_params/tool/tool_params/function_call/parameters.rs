use serde::Deserialize;
use serde::Serialize;

use super::parameters_schema::ParametersSchema;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
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
