use serde::Deserialize;
use serde::Serialize;

use crate::agent_desired_state::AgentDesiredState;

#[derive(Default, Deserialize, Serialize)]
pub struct Schema {
    pub agent_desired_state: AgentDesiredState,
}
