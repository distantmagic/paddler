use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct UpstreamPeer {
    pub agent_name: Option<String>,
    pub error: Option<String>,
    pub slots_idle: usize,
    pub slots_processing: usize,
}
