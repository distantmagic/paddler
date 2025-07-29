use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub description: String,
}
