use serde::Deserialize;
use serde::Serialize;

use crate::decision_settings::DecisionSettings;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum AgentDecisionsSettings {
    NotServed,
    Served(DecisionSettings),
}
