pub mod agent;
pub mod balancer;
pub mod handler;

use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;

fn resolve_socket_addr(s: &str) -> Result<SocketAddr> {
    let addrs: Vec<SocketAddr> = s.to_socket_addrs()?.collect();

    for addr in &addrs {
        if addr.is_ipv4() {
            return Ok(*addr);
        }
    }

    for addr in addrs {
        if addr.is_ipv6() {
            return Ok(addr);
        }
    }

    Err(anyhow!("Failed to resolve socket address"))
}

fn parse_duration(arg: &str) -> Result<Duration> {
    let milliseconds = arg.parse()?;

    Ok(std::time::Duration::from_millis(milliseconds))
}

fn parse_socket_addr(arg: &str) -> Result<SocketAddr> {
    match arg.parse() {
        Ok(socketaddr) => Ok(socketaddr),
        Err(_) => Ok(resolve_socket_addr(arg)?),
    }
}
