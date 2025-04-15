#[cfg(test)]
mod tests {
    use crate::{
        balancer::{status_update::StatusUpdate, upstream_peer_pool::UpstreamPeerPool},
        errors::result::Result,
        llamacpp::slot::Slot,
    };

    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        time::SystemTime,
    };

    fn create_update(
        agent_name: String,
        agent_addr: SocketAddr,
        slots: usize,
        processing: bool,
    ) -> Result<StatusUpdate> {
        let error = None;
        let is_authorized = Some(true);
        let is_slots_endpoint_enabled = Some(true);

        let slots = (1..=slots)
            .map(|id| Slot {
                id,
                is_processing: processing,
            })
            .collect();

        Ok(StatusUpdate::new(
            Some(agent_name),
            error.clone(),
            agent_addr,
            is_authorized,
            is_slots_endpoint_enabled,
            slots,
        ))
    }

    #[test]
    fn upstream_peer_is_registered() -> Result<()> {
        let upstream_peer_pool = UpstreamPeerPool::new();

        let agent_name = "agent-1".to_string();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let agent_id = "a8a6d626-23be-40ad-9ee3-7f960e56f130";

        upstream_peer_pool
            .register_status_update(agent_id, create_update(agent_name.clone(), addr, 1, true)?)?;

        let agents = upstream_peer_pool.agents.read()?;

        assert_eq!(agents[0].agent_name, Some(agent_name));
        assert_eq!(agents[0].agent_id, agent_id);
        assert_eq!(agents[0].error, None);
        assert_eq!(agents[0].external_llamacpp_addr, addr);
        assert_eq!(agents[0].is_authorized, Some(true));
        assert_eq!(agents[0].is_slots_endpoint_enabled, Some(true));

        Ok(())
    }

    #[test]
    fn peer_is_removed() -> Result<()> {
        let upstream_peer_pool = UpstreamPeerPool::new();
        let agent_id = "a8a6d626-23be-40ad-9ee3-7f960e56f130";

        upstream_peer_pool.register_status_update(
            agent_id,
            create_update(
                "agent-1".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
                1,
                true,
            )?,
        )?;

        upstream_peer_pool.remove_peer(agent_id)?;

        let agents = upstream_peer_pool.agents.read()?.clone();

        assert!(agents.get(0).is_none());

        Ok(())
    }

    #[test]
    fn integrity_is_restored() -> Result<()> {
        let upstream_peer_pool = UpstreamPeerPool::new();
        let agent_name = "agent-2".to_string();

        upstream_peer_pool.register_status_update(
            "a8a6d626-23be-40ad-9ee3-7f960e56f130",
            create_update(
                "agent-1".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
                4,
                true,
            )?,
        )?;

        upstream_peer_pool.register_status_update(
            "a8a6d626-23be-40ad-9ee3-7f960e56f131",
            create_update(
                agent_name.clone(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                4,
                false,
            )?,
        )?;

        let agents = upstream_peer_pool.agents.read()?.clone();

        assert!(agents[0].agent_name == Some(agent_name));

        Ok(())
    }

    #[test]
    fn peer_is_quarantined() -> Result<()> {
        let upstream_peer_pool = UpstreamPeerPool::new();
        let agent_id = "a8a6d626-23be-40ad-9ee3-7f960e56f130";

        upstream_peer_pool.register_status_update(
            agent_id,
            create_update(
                "agent-1".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
                1,
                true,
            )?,
        )?;

        let result = upstream_peer_pool.quarantine_peer(agent_id)?;

        let agents = upstream_peer_pool.agents.read()?.clone();

        assert!(result);
        assert!(agents[0].quarantined_until.unwrap() > SystemTime::now());

        Ok(())
    }

    #[cfg(feature = "statsd_reporter")]
    #[test]
    fn total_slots_is_used() -> Result<()> {
        let upstream_peer_pool = UpstreamPeerPool::new();

        upstream_peer_pool.register_status_update(
            "a8a6d626-23be-40ad-9ee3-7f960e56f130",
            create_update(
                "agent-1".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
                4,
                false,
            )?,
        )?;

        let (slots_idle, slots_processing) = upstream_peer_pool.total_slots()?;

        assert_eq!(slots_idle, 4);
        assert_eq!(slots_processing, 0);

        Ok(())
    }

    #[test]
    fn best_peer_is_used() -> Result<()> {
        let upstream_peer_pool = UpstreamPeerPool::new();

        upstream_peer_pool.register_status_update(
            "a8a6d626-23be-40ad-9ee3-7f960e56f130",
            create_update(
                "agent-1".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
                4,
                true,
            )?,
        )?;

        upstream_peer_pool.register_status_update(
            "a8a6d626-23be-40ad-9ee3-7f960e56f131",
            create_update(
                "agent-2".to_string(),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                4,
                false,
            )?,
        )?;

        assert_eq!(
            upstream_peer_pool.use_best_peer()?.unwrap().agent_name,
            Some("agent-2".to_string())
        );

        Ok(())
    }
}
