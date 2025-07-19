use serde::Deserialize;
use serde::Serialize;

use crate::response_params::GeneratedToken;

#[derive(Deserialize, Serialize)]
pub enum Response {
    GeneratedToken(GeneratedToken),
}
