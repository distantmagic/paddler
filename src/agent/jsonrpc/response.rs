use serde::Deserialize;
use serde::Serialize;

use super::response_params::GeneratedToken;

#[derive(Deserialize, Serialize)]
pub enum Response {
    GeneratedToken(GeneratedToken),
}
