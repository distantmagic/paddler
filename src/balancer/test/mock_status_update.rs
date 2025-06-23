use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;

use crate::balancer::status_update::StatusUpdate;
use crate::llamacpp::slot::Slot;

pub fn mock_status_update(
    agent_id: &str,
    slots_idle: usize,
    slots_processing: usize,
) -> StatusUpdate {
    let mut i = 0;
    let mut slots = Vec::new();

    for _ in 0..slots_idle {
        slots.push(Slot {
            id: i,
            is_processing: false,
        });

        i += 1;
    }

    for _ in 0..slots_processing {
        slots.push(Slot {
            id: i,
            is_processing: true,
        });

        i += 1;
    }

    let idle_slots_count = slots.iter().filter(|slot| !slot.is_processing).count();

    StatusUpdate {
        agent_name: Some(agent_id.to_string()),
        error: None,
        is_unexpected_response_status: None,
        is_connect_error: None,
        is_decode_error: None,
        is_deserialize_error: None,
        is_request_error: None,
        external_llamacpp_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        idle_slots_count,
        is_authorized: Some(true),
        is_slots_endpoint_enabled: Some(true),
        processing_slots_count: slots.len() - idle_slots_count,
    }
}
