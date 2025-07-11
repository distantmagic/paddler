use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct GeneratedToken {
    pub token: String,
}
