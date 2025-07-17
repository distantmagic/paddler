mod agent;
mod atomic_value;
mod balancer;
mod cmd;
mod controls_websocket_endpoint;
mod create_cors_middleware;
mod database_type;
mod jsonrpc;
mod produces_snapshot;
mod request_params;
mod response_params;
mod rpc_message;
mod sends_serialized_message;
mod service;
mod service_manager;
#[cfg(feature = "web_dashboard")]
mod static_files;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
#[cfg(feature = "web_dashboard")]
use esbuild_metafile::instance::initialize_instance;
use log::info;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tokio::sync::oneshot;

use crate::cmd::agent::Agent;
use crate::cmd::balancer::Balancer;
use crate::cmd::handler::Handler as _;

#[cfg(feature = "web_dashboard")]
pub const ESBUILD_META_CONTENTS: &str = include_str!("../esbuild-meta.json");

#[derive(Parser)]
#[command(arg_required_else_help(true), version, about, long_about = None)]
/// Stateful load balancer for llama.cpp
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Agent for managing llama.cpp instances
    Agent(Agent),
    /// Balances incoming requests to llama.cpp instances and optionally provides a web dashboard
    Balancer(Balancer),
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
        Some(Commands::Agent(handler)) => handler.handle(shutdown_rx).await,
        Some(Commands::Balancer(handler)) => {
            #[cfg(feature = "web_dashboard")]
            initialize_instance(ESBUILD_META_CONTENTS);

            handler.handle(shutdown_rx).await
        }
        None => Ok(()),
    }
}
