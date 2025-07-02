use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {}
