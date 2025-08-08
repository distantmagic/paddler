mod function;
mod parameters;
mod parameters_schema;

use serde::Deserialize;
use serde::Serialize;

use self::function::Function;

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionCall {
    pub function: Function,
}
