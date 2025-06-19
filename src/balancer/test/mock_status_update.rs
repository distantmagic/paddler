// use std::net::IpAddr;
// use std::net::Ipv4Addr;
// use std::net::SocketAddr;

// use crate::balancer::status_update::StatusUpdate;
// use crate::llamacpp::slot::Slot;

// pub fn mock_status_update(
//     agent_id: &str,
//     slots_idle: usize,
//     slots_processing: usize,
// ) -> StatusUpdate {
//     let mut i = 0;
//     let mut slots = Vec::new();

//     for _ in 0..slots_idle {
//         slots.push(Slot {
//             id: i,
//             is_processing: false,
//         });

//         i += 1;
//     }

//     for _ in 0..slots_processing {
//         slots.push(Slot {
//             id: i,
//             is_processing: true,
//         });

//         i += 1;
//     }

//     StatusUpdate::new(
//         Some(agent_id.to_string()),
//         None,
//         SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
//         Some(true),
//         Some(true),
//         slots,
//     )
// }
