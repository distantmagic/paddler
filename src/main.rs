mod agent;
mod balancer;
mod cmd;
mod errors;
mod llamacpp;
#[cfg(feature = "web_dashboard")]
mod static_files;
mod supervisor;

use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::time::Duration;

use clap::Parser;
use clap::Subcommand;
#[cfg(feature = "web_dashboard")]
use esbuild_metafile::instance::initialize_instance;

use crate::errors::result::Result;

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

        #[arg(long, default_value = "10", value_parser = parse_duration)]
        /// Interval (in seconds) at which the agent will report the status of the llama.cpp instance
        monitoring_interval: Duration,

        #[arg(long)]
        /// Name of the agent (optional)
        name: Option<String>,
    },
    /// Balances incoming requests to llama.cpp instances and optionally provides a web dashboard
    Balancer {
        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the management server that the balancer will report to
        management_addr: SocketAddr,

        #[cfg(feature = "web_dashboard")]
        #[arg(long)]
        /// Enable the web management dashboard
        management_dashboard_enable: bool,

        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the reverse proxy server
        reverseproxy_addr: SocketAddr,

        #[arg(long)]
        /// Rewrite the host header of incoming requests so that it matches the upstream server
        /// instead of the reverse client server
        rewrite_host_header: bool,

        #[arg(long, default_value = "30", value_parser = parse_duration)]
        /// The request timeout (in seconds). For all requests that a timely response from an
        /// upstream isn't received for, the 504 (Gateway Timeout) error is issued.
        request_timeout: Duration,

        #[arg(long, default_value = "32")]
        /// The maximum number of buffered requests. Like with usual requests, the request timeout
        /// is also applied to buffered ones. If the maximum number is reached, all new requests are
        /// rejected with the 429 (Too Many Requests) error.
        max_requests: usize,

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
        #[arg(long, default_value = "10", value_parser = parse_duration)]
        /// Interval (in seconds) at which the balancer will report metrics to statsd
        statsd_reporting_interval: Duration,
    },
    #[cfg(feature = "ratatui_dashboard")]
    /// Command-line dashboard for monitoring the balancer
    Dashboard {
        #[arg(long, value_parser = parse_socket_addr)]
        /// Address of the management server that the dashboard will connect to
        management_addr: SocketAddr,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match &cli.command {
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
            management_addr,
            #[cfg(feature = "web_dashboard")]
            management_dashboard_enable,
            reverseproxy_addr,
            rewrite_host_header,
            slots_endpoint_enable,
            #[cfg(feature = "statsd_reporter")]
            statsd_addr,
            #[cfg(feature = "statsd_reporter")]
            statsd_prefix,
            #[cfg(feature = "statsd_reporter")]
            statsd_reporting_interval,
            request_timeout,
            max_requests,
        }) => {
            #[cfg(feature = "web_dashboard")]
            initialize_instance(ESBUILD_META_CONTENTS);

            cmd::balancer::handle(
                management_addr,
                #[cfg(feature = "web_dashboard")]
                management_dashboard_enable.to_owned(),
                reverseproxy_addr,
                rewrite_host_header.to_owned(),
                slots_endpoint_enable.to_owned(),
                #[cfg(feature = "statsd_reporter")]
                statsd_addr.to_owned(),
                #[cfg(feature = "statsd_reporter")]
                statsd_prefix.to_owned(),
                #[cfg(feature = "statsd_reporter")]
                statsd_reporting_interval.to_owned(),
                *request_timeout,
                *max_requests,
            )
        }
        #[cfg(feature = "ratatui_dashboard")]
        Some(Commands::Dashboard {
            management_addr,
        }) => cmd::dashboard::handle(management_addr),
        None => Ok(()),
    }
}
