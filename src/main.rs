mod balancer;
mod cmd;
mod jsonrpc;
mod llamacpp;
mod service;
mod service_manager;
#[cfg(feature = "web_dashboard")]
mod static_files;
mod supervisor;

use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
#[cfg(feature = "web_dashboard")]
use esbuild_metafile::instance::initialize_instance;
use log::info;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tokio::sync::oneshot;

use crate::balancer::fleet_management_database::File;
use crate::balancer::fleet_management_database::Memory;
use crate::balancer::fleet_management_database_type::FleetManagementDatabaseType;
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
    /// Balances incoming requests to llama.cpp instances and optionally provides a web dashboard
    Balancer {
        #[arg(long, default_value = "10000", value_parser = parse_duration)]
        /// The request timeout (in milliseconds). For all requests that a timely response from an
        /// upstream isn't received for, the 504 (Gateway Timeout) error is issued.
        buffered_request_timeout: Duration,

        #[arg(long, default_value = "memory://")]
        // Fleet management database URL. Supported: memory, memory://, or file:///path (optional)
        fleet_management_database: FleetManagementDatabaseType,

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

        #[arg(long, default_value = "30")]
        /// The maximum number of buffered requests. Like with usual requests, the request timeout
        /// is also applied to buffered ones. If the maximum number is reached, all new requests are
        /// rejected with the 429 (Too Many Requests) error.
        max_buffered_requests: usize,

        #[arg(long, default_value = "127.0.0.1:8061", value_parser = parse_socket_addr)]
        /// Address of the reverse proxy server
        reverseproxy_addr: SocketAddr,

        #[arg(long)]
        /// Enable the web metrics endpoint
        metrics_endpoint_enable: bool,

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
    /// Supervisor for managing llama.cpp instances
    Supervisor {
        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the llama.cpp instance that the supervisor will spawn and manage
        llamacpp_listen_addr: SocketAddr,

        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the management server that the supervisor will report to
        management_addr: SocketAddr,

        #[arg(long)]
        /// Name of the supervisor (optional)
        name: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to listen for SIGTERM");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to listen for SIGINT");
        let mut sighup = signal(SignalKind::hangup()).expect("Failed to listen for SIGHUP");

        tokio::select! {
            _ = sigterm.recv() => info!("Received SIGTERM"),
            _ = sigint.recv() => info!("Received SIGINT (Ctrl+C)"),
            _ = sighup.recv() => info!("Received SIGHUP"),
        }

        shutdown_tx
            .send(())
            .expect("Failed to send shutdown signal");
    });

    match Cli::parse().command {
        Some(Commands::Balancer {
            buffered_request_timeout,
            fleet_management_database,
            fleet_management_enable,
            management_addr,
            management_cors_allowed_hosts,
            max_buffered_requests,
            metrics_endpoint_enable,
            reverseproxy_addr,
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
        }) => {
            #[cfg(feature = "web_dashboard")]
            initialize_instance(ESBUILD_META_CONTENTS);

            cmd::balancer::handle(
                buffered_request_timeout,
                match fleet_management_database {
                    FleetManagementDatabaseType::File(path) => Arc::new(File::new(path)),
                    FleetManagementDatabaseType::Memory => Arc::new(Memory::new()),
                },
                ManagementServiceConfiguration {
                    addr: management_addr,
                    cors_allowed_hosts: management_cors_allowed_hosts,
                    fleet_management_enable,
                    metrics_endpoint_enable,
                },
                max_buffered_requests,
                reverseproxy_addr,
                shutdown_rx,
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
            .await
        }
        Some(Commands::Supervisor {
            llamacpp_listen_addr,
            management_addr,
            name,
        }) => {
            cmd::supervisor::handle(llamacpp_listen_addr, management_addr, name, shutdown_rx).await
        }
        None => Ok(()),
    }
}
