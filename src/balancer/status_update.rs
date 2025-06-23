use std::net::SocketAddr;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub agent_name: Option<String>,
    pub error: Option<String>,
    pub external_llamacpp_addr: SocketAddr,
    pub idle_slots_count: usize,
    pub is_authorized: Option<bool>,
    pub is_connect_error: Option<bool>,
    pub is_decode_error: Option<bool>,
    pub is_deserialize_error: Option<bool>,
    pub is_request_error: Option<bool>,
    pub is_slots_endpoint_enabled: Option<bool>,
    pub is_unexpected_response_status: Option<bool>,
    pub processing_slots_count: usize,
}

impl actix::Message for StatusUpdate {
    type Result = ();
}
