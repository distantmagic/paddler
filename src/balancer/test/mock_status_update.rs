use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;

use crate::balancer::status_update::StatusUpdate;

pub fn mock_status_update(
    agent_id: &str,
    slots_idle: usize,
    slots_processing: usize,
) -> StatusUpdate {
    StatusUpdate {
        agent_name: Some(agent_id.to_string()),
        error: None,
        external_llamacpp_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        is_authorized: Some(true),
        is_connect_error: Some(false),
        is_decode_error: Some(false),
        is_deserialize_error: Some(false),
        is_request_error: Some(false),
        is_slots_endpoint_enabled: Some(true),
        is_unexpected_response_status: Some(false),
        slots_idle,
        slots_processing,
    }
}
