use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use crate::chat_template::ChatTemplate;
use crate::agent_desired_state::AgentDesiredState;

#[derive(Default, Deserialize, Serialize)]
pub struct Schema {
    pub agent_desired_state: AgentDesiredState,
    pub chat_templates: BTreeMap<String, ChatTemplate>,
}
