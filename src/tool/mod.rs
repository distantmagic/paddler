mod tool_params;

use serde::Deserialize;
use serde::Serialize;

use self::tool_params::FunctionCall;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Tool {
    Function(FunctionCall),
}
