mod agent;
mod balancer;
mod cmd;
#[cfg(feature = "supervisor")]
mod jsonrpc;
mod llamacpp;
#[cfg(feature = "web_dashboard")]
mod static_files;
#[cfg(feature = "supervisor")]
mod supervisor;

use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
#[cfg(feature = "web_dashboard")]
use esbuild_metafile::instance::initialize_instance;

use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
#[cfg(feature = "statsd_reporter")]
use crate::balancer::statsd_service::configuration::Configuration as StatsdServiceConfiguration;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;

#[cfg(feature = "web_dashboard")]
pub const ESBUILD_META_CONTENTS: &str = include_str!("../esbuild-meta.json");

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
        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of llama.cpp instance that the balancer will forward requests to. If not
        /// provided, then `--local-llamacpp-addr` will be used
        external_llamacpp_addr: Option<SocketAddr>,

        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the local llama.cpp instance that the agent will monitor
        local_llamacpp_addr: SocketAddr,

        #[arg(long)]
        /// API key for the llama.cpp instance (optional)
        llamacpp_api_key: Option<String>,

        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the management server that the agent will report to
        management_addr: SocketAddr,

        #[arg(long, default_value = "10000", value_parser = parse_duration)]
        /// Interval (in milliseconds) at which the agent will report the status of the llama.cpp instance
        monitoring_interval: Duration,

        #[arg(long)]
        /// Name of the agent (optional)
        name: Option<String>,
    },
    /// Balances incoming requests to llama.cpp instances and optionally provides a web dashboard
    Balancer {
        #[arg(long, default_value = "10000", value_parser = parse_duration)]
        /// The request timeout (in milliseconds). For all requests that a timely response from an
        /// upstream isn't received for, the 504 (Gateway Timeout) error is issued.
        buffered_request_timeout: Duration,

        #[cfg(feature = "supervisor")]
        #[arg(long)]
        // Path to the fleet database file. If not exists, it will be created.
        fleet_database_path: Option<PathBuf>,

        #[cfg(feature = "supervisor")]
        #[arg(long)]
        /// Enable registering supervisor-managed llama.cpp instances in the balancer
        fleet_management_enable: bool,

        #[arg(long, default_value = "127.0.0.1:8060", value_parser = parse_socket_addr)]
        /// Address of the management server that the balancer will report to
        management_addr: SocketAddr,

        #[arg(
            long = "management-cors-allowed-host",
            help = "Allowed CORS host (can be specified multiple times)",
            action = clap::ArgAction::Append
        )]
        management_cors_allowed_hosts: Vec<String>,

        #[cfg(feature = "web_dashboard")]
        #[arg(long)]
        /// Enable the web management dashboard
        management_dashboard_enable: bool,

        #[arg(long, default_value = "30")]
        /// The maximum number of buffered requests. Like with usual requests, the request timeout
        /// is also applied to buffered ones. If the maximum number is reached, all new requests are
        /// rejected with the 429 (Too Many Requests) error.
        max_buffered_requests: usize,

        #[arg(long, default_value = "127.0.0.1:8061", value_parser = parse_socket_addr)]
        /// Address of the reverse proxy server
        reverseproxy_addr: SocketAddr,

        #[arg(long)]
        /// Rewrite the host header of incoming requests so that it matches the upstream server
        /// instead of the reverse client server
        rewrite_host_header: bool,

        #[arg(long)]
        /// Enable the slots endpoint (not recommended)
        slots_endpoint_enable: bool,

        #[cfg(feature = "statsd_reporter")]
        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the statsd server to report metrics to
        statsd_addr: Option<SocketAddr>,

        #[cfg(feature = "statsd_reporter")]
        #[arg(long, default_value = "paddler")]
        /// Prefix for statsd metrics
        statsd_prefix: String,

        #[cfg(feature = "statsd_reporter")]
        #[arg(long, default_value = "10000", value_parser = parse_duration)]
        /// Interval (in milliseconds) at which the balancer will report metrics to statsd
        statsd_reporting_interval: Duration,

        #[arg(long, default_value = "127.0.0.1:8061", value_parser = parse_socket_addr)]
        /// Address of the web management dashboard (if enabled)
        web_dashboard_addr: Option<SocketAddr>,

        #[cfg(feature = "web_dashboard")]
        #[arg(long, default_value = "false")]
        /// Enable the web management dashboard
        web_dashboard_enable: bool,
    },
    #[cfg(feature = "ratatui_dashboard")]
    /// Command-line dashboard for monitoring the balancer
    Dashboard {
        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the management server that the dashboard will connect to
        management_addr: SocketAddr,
    },
    #[cfg(feature = "supervisor")]
    /// Supervisor for managing llama.cpp instances
    Supervisor {
        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the management server that the supervisor will report to
        management_addr: SocketAddr,

        #[arg(long)]
        /// Name of the supervisor (optional)
        name: Option<String>,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    match Cli::parse().command {
        Some(Commands::Agent {
            external_llamacpp_addr,
            local_llamacpp_addr,
            llamacpp_api_key,
            management_addr,
            monitoring_interval,
            name,
        }) => cmd::agent::handle(
            match external_llamacpp_addr {
                Some(addr) => addr.to_owned(),
                None => local_llamacpp_addr.to_owned(),
            },
            local_llamacpp_addr.to_owned(),
            llamacpp_api_key.to_owned(),
            management_addr.to_owned(),
            monitoring_interval.to_owned(),
            name.to_owned(),
        ),
        Some(Commands::Balancer {
            buffered_request_timeout,
            management_addr,
            management_cors_allowed_hosts,
            max_buffered_requests,
            reverseproxy_addr,
            rewrite_host_header,
            slots_endpoint_enable,
            #[cfg(feature = "statsd_reporter")]
            statsd_addr,
            #[cfg(feature = "statsd_reporter")]
            statsd_prefix,
            #[cfg(feature = "statsd_reporter")]
            statsd_reporting_interval,
            #[cfg(feature = "web_dashboard")]
            web_dashboard_addr,
            #[cfg(feature = "web_dashboard")]
            web_dashboard_enable,
            ..
        }) => {
            #[cfg(feature = "web_dashboard")]
            initialize_instance(ESBUILD_META_CONTENTS);

            cmd::balancer::handle(
                buffered_request_timeout,
                ManagementServiceConfiguration {
                    addr: management_addr,
                    cors_allowed_hosts: management_cors_allowed_hosts,
                },
                max_buffered_requests,
                reverseproxy_addr,
                rewrite_host_header.to_owned(),
                slots_endpoint_enable.to_owned(),
                #[cfg(feature = "statsd_reporter")]
                statsd_addr.map(|statsd_addr| StatsdServiceConfiguration {
                    statsd_addr,
                    statsd_prefix,
                    statsd_reporting_interval,
                }),
                #[cfg(feature = "web_dashboard")]
                if web_dashboard_enable {
                    web_dashboard_addr.map(|web_dashboard_addr| WebDashboardServiceConfiguration {
                        addr: web_dashboard_addr,
                        management_addr,
                    })
                } else {
                    None
                },
            )
        }
        #[cfg(feature = "ratatui_dashboard")]
        Some(Commands::Dashboard {
            management_addr,
        }) => cmd::dashboard::handle(management_addr),
        #[cfg(feature = "supervisor")]
        Some(Commands::Supervisor {
            management_addr,
            name,
        }) => cmd::supervisor::handle(management_addr, name),
        None => Ok(()),
    }
}
