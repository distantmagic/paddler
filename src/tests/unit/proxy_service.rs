#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
    };

    use crate::{
        balancer::{
            proxy_service::{LlamaCppContext, ProxyService},
            status_update::StatusUpdate,
            upstream_peer::UpstreamPeer,
            upstream_peer_pool::UpstreamPeerPool,
        },
        errors::result::Result,
        llamacpp::slot::Slot,
    };

    #[tokio::test]
    async fn slot_is_freed() -> Result<()> {
        let upstream_peer = UpstreamPeer::new(
            "a8a6d626-23be-40ad-9ee3-7f960e56f130".to_string(),
            None,
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            Some(true),
            Some(true),
            8,
            0,
        );

        let mut ctx = LlamaCppContext {
            slot_taken: true,
            selected_peer: Some(upstream_peer.clone()),
            uses_slots: true,
        };

        let upstream_peer_pool = UpstreamPeerPool::new();
        upstream_peer_pool.register_status_update(
            &upstream_peer.clone().agent_id,
            StatusUpdate::new(
                upstream_peer.clone().agent_name,
                upstream_peer.clone().error,
                upstream_peer.clone().external_llamacpp_addr,
                upstream_peer.clone().is_authorized,
                upstream_peer.clone().is_slots_endpoint_enabled,
                vec![Slot {
                    id: 1,
                    is_processing: true,
                }],
            ),
        )?;

        let proxy_service = ProxyService::new(false, true, upstream_peer_pool.into());

        proxy_service.release_slot(&mut ctx)?;

        assert!(!ctx.slot_taken);
        assert_eq!(upstream_peer.clone().slots_idle, 8);
        assert_eq!(upstream_peer.slots_processing, 0);

        Ok(())
    }

    #[tokio::test]
    async fn slot_is_taken() -> Result<()> {
        let agent_name = None;
        let agent_id = "a8a6d626-23be-40ad-9ee3-7f960e56f130".to_string();
        let error = None;
        let external_llamacpp_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let is_authorized = Some(true);
        let is_slots_endpoint_enabled = Some(true);

        let upstream_peer = UpstreamPeer::new(
            agent_id.clone(),
            agent_name.clone(),
            error.clone(),
            external_llamacpp_addr,
            Some(true),
            Some(true),
            8,
            0,
        );

        let mut ctx = LlamaCppContext {
            slot_taken: false,
            selected_peer: Some(upstream_peer.clone()),
            uses_slots: true,
        };

        let upstream_peer_pool = Arc::new(UpstreamPeerPool::new());

        let slots: Vec<Slot> = (1..=8)
            .map(|id| Slot {
                id,
                is_processing: false,
            })
            .collect();

        upstream_peer_pool.register_status_update(
            &agent_id,
            StatusUpdate::new(
                agent_name,
                error,
                external_llamacpp_addr,
                is_authorized,
                is_slots_endpoint_enabled,
                slots,
            ),
        )?;

        let proxy_service = ProxyService::new(false, true, upstream_peer_pool.clone());

        proxy_service.take_slot(&mut ctx)?;

        let agents = upstream_peer_pool.agents.read().unwrap();
        let updated_peer = agents.iter().find(|p| p.agent_id == agent_id).unwrap();

        assert!(ctx.slot_taken);
        assert_eq!(updated_peer.slots_idle, 7);
        assert_eq!(updated_peer.slots_processing, 1);

        Ok(())
    }
}
