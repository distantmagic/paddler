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
#[command(arg_required_else_help(true), version, about, long_about = None)]
/// Stateful load balancer for llama.cpp
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Monitors llama.cpp instance and reports their status to the balancer
    Agent {
        /// Address of llama.cpp instance that the balancer will forward requests to. If not
        /// provided, then `--local-llamacpp-addr` will be used
        #[arg(long, value_parser = parse_socket_addr)]
        external_llamacpp_addr: Option<SocketAddr>,

        /// Address of the local llama.cpp instance that the agent will monitor
        #[arg(long, value_parser = parse_socket_addr)]
        local_llamacpp_addr: SocketAddr,

        #[arg(long)]
        /// API key for the llama.cpp instance
        llamacpp_api_key: Option<String>,

        #[arg(long, value_parser = parse_socket_addr)]
        management_addr: SocketAddr,

        #[arg(long, default_value = "10", value_parser = parse_duration)]
        monitoring_interval: Duration,

        #[arg(long)]
        name: Option<String>,
    },
    /// Balances incoming requests to llama.cpp instances and optionally provides a web dashboard
    Balancer {
        #[arg(long, value_parser = parse_socket_addr)]
        management_addr: SocketAddr,

        #[arg(long)]
        management_dashboard_enable: bool,

        #[arg(long, value_parser = parse_socket_addr)]
        reverseproxy_addr: SocketAddr,

        #[arg(long)]
        rewrite_host_header: bool,

        #[arg(long)]
        slots_endpoint_enable: bool,

        #[arg(long, value_parser = parse_socket_addr)]
        statsd_addr: Option<SocketAddr>,

        #[arg(long, default_value = "paddler")]
        statsd_prefix: String,

        #[arg(long, default_value = "10", value_parser = parse_duration)]
        statsd_reporting_interval: Duration,
    },
    // Command-line dashboard for monitoring the balancer
    // Dashboard {
    //     #[arg(long, value_parser = parse_socket_addr)]
    //     management_addr: SocketAddr,
    // },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Agent {
            external_llamacpp_addr,
            local_llamacpp_addr,
            llamacpp_api_key,
            management_addr,
            monitoring_interval,
            name,
        }) => {
            cmd::agent::handle(
                match external_llamacpp_addr {
                    Some(addr) => addr.to_owned(),
                    None => local_llamacpp_addr.to_owned(),
                },
                local_llamacpp_addr.to_owned(),
                llamacpp_api_key.to_owned(),
                management_addr.to_owned(),
                monitoring_interval.to_owned(),
                name.to_owned(),
            )?;
        }
        Some(Commands::Balancer {
            management_addr,
            management_dashboard_enable,
            reverseproxy_addr,
            rewrite_host_header,
            slots_endpoint_enable,
            statsd_addr,
            statsd_prefix,
            statsd_reporting_interval,
        }) => {
            cmd::balancer::handle(
                management_addr,
                management_dashboard_enable.to_owned(),
                reverseproxy_addr,
                rewrite_host_header.to_owned(),
                slots_endpoint_enable.to_owned(),
                statsd_addr.to_owned(),
                statsd_prefix.to_owned(),
                statsd_reporting_interval.to_owned(),
            )?;
        }
        // Some(Commands::Dashboard { management_addr }) => {}
        None => {}
    }

    Ok(())
}
