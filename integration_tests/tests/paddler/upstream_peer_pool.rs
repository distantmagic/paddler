use serde::Deserialize;

use crate::upstream_peer::UpstreamPeer;

#[derive(Deserialize, Debug)]
pub struct UpstreamPeerPool {
    pub agents: Vec<UpstreamPeer>,
}
