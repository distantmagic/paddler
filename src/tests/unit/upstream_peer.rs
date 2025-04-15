#[cfg(test)]
mod tests {
    use crate::{
        balancer::{
            status_update::StatusUpdate, upstream_peer::UpstreamPeer,
            upstream_peer_pool::UpstreamPeerPool,
        },
        errors::result::Result,
        llamacpp::slot::Slot,
    };

    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        time::SystemTime,
    };

    #[test]
    fn upstream_peer_is_usable() -> Result<()> {
        let upstream_peer = UpstreamPeer::new(
            "a8a6d626-23be-40ad-9ee3-7f960e56f130".to_string(),
            Some("agent-1".to_string()),
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            Some(true),
            None,
            5,
            0,
        );

        assert_eq!(upstream_peer.is_usable(), true);

        Ok(())
    }

    #[test]
    fn upstream_peer_is_not_usable() -> Result<()> {
        let upstream_peer = UpstreamPeer::new(
            "a8a6d626-23be-40ad-9ee3-7f960e56f130".to_string(),
            Some("agent-1".to_string()),
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            None,
            None,
            5,
            0,
        );

        assert_eq!(upstream_peer.is_usable(), false);

        Ok(())
    }

    #[test]
    fn status_is_updated() -> Result<()> {
        let mut upstream_peer = UpstreamPeer::new(
            "a8a6d626-23be-40ad-9ee3-7f960e56f130".to_string(),
            Some("agent-1".to_string()),
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            None,
            None,
            5,
            0,
        );

        let update = StatusUpdate::new(
            Some("agent-2".to_string()),
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            None,
            None,
            vec![],
        );

        upstream_peer.update_status(update);

        assert_ne!(upstream_peer.agent_name, Some("agent-1".to_string()));
        assert_ne!(upstream_peer.slots_idle, 5);
        assert_ne!(
            upstream_peer.external_llamacpp_addr,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081)
        );

        assert_eq!(upstream_peer.agent_name, Some("agent-2".to_string()));
        assert_eq!(upstream_peer.slots_idle, 0);
        assert_eq!(
            upstream_peer.external_llamacpp_addr,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
        );

        Ok(())
    }
}
