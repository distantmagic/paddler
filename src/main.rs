use clap::{Parser, Subcommand};
use std::{
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};

use crate::errors::result::Result;

mod agent;
mod balancer;
mod cmd;
mod errors;
mod llamacpp;

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

    Err("Failed to resolve socket address".into())
}

fn parse_duration(arg: &str) -> Result<Duration> {
    let seconds = arg.parse()?;

    Ok(std::time::Duration::from_secs(seconds))
}

fn parse_socket_addr(arg: &str) -> Result<SocketAddr> {
    match arg.parse() {
        Ok(socketaddr) => Ok(socketaddr),
        Err(_) => Ok(resolve_socket_addr(arg)?),
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Agent {
        #[arg(long, value_parser = parse_socket_addr)]
        external_llamacpp_addr: SocketAddr,

        #[arg(long, value_parser = parse_socket_addr)]
        local_llamacpp_addr: SocketAddr,

        #[arg(long)]
        local_llamacpp_api_key: Option<String>,

        #[arg(long, value_parser = parse_socket_addr)]
        management_addr: SocketAddr,

        #[arg(long, default_value = "20", value_parser = parse_duration)]
        monitoring_interval: Duration,

        #[arg(long)]
        name: Option<String>,
    },
    Balancer {
        #[arg(long, value_parser = parse_socket_addr)]
        management_addr: SocketAddr,

        #[arg(long)]
        management_dashboard_enable: bool,

        #[arg(long, value_parser = parse_socket_addr)]
        reverseproxy_addr: SocketAddr,

        #[arg(long)]
        rewrite_host_header: bool,
    },
    Dashboard {
        #[arg(long, value_parser = parse_socket_addr)]
        management_addr: SocketAddr,
    },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Agent {
            external_llamacpp_addr,
            local_llamacpp_addr,
            local_llamacpp_api_key,
            management_addr,
            monitoring_interval,
            name,
        }) => {
            cmd::agent::handle(
                external_llamacpp_addr.clone(),
                local_llamacpp_addr.clone(),
                local_llamacpp_api_key.clone(),
                management_addr.clone(),
                monitoring_interval.clone(),
                name.clone(),
            )?;
        }
        Some(Commands::Balancer {
            management_addr,
            management_dashboard_enable,
            reverseproxy_addr,
            rewrite_host_header,
        }) => {
            cmd::balancer::handle(
                management_addr,
                management_dashboard_enable.clone(),
                reverseproxy_addr,
                rewrite_host_header.clone(),
            )?;
        }
        Some(Commands::Dashboard { management_addr }) => {}
        None => {}
    }

    Ok(())
}
