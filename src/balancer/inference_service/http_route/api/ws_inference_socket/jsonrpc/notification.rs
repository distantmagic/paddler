use serde::Deserialize;
use serde::Serialize;

use super::notification_params::GenerateTokensParams;

#[derive(Deserialize, Serialize)]
#[serde(tag = "notification", content = "content")]
pub enum Notification {
    GenerateTokens(GenerateTokensParams),
}
