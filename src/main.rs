mod agent;
mod agent_applicable_state;
mod agent_applicable_state_holder;
mod agent_desired_model;
mod agent_desired_state;
mod agent_issue;
mod agent_issue_fix;
mod atomic_value;
mod chat_template;
mod balancer;
mod cmd;
mod controls_websocket_endpoint;
mod conversation_message;
mod converts_to_applicable_state;
mod create_cors_middleware;
mod dispenses_slots;
mod generated_token;
mod generated_token_envelope;
mod generated_token_result;
mod huggingface_model_reference;
mod inference_parameters;
mod jsonrpc;
mod model_metadata;
mod produces_snapshot;
mod request_params;
mod rpc_message;
mod sends_rpc_message;
mod service;
mod service_manager;
mod sets_desired_state;
mod slot_aggregated_status;
mod slot_aggregated_status_download_progress;
mod slot_aggregated_status_manager;
mod slot_aggregated_status_snapshot;
mod slot_status;
#[cfg(feature = "web_admin_panel")]
mod static_files;
mod tool;
mod websocket_session_controller;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
#[cfg(feature = "web_admin_panel")]
use esbuild_metafile::instance::initialize_instance;
use log::info;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tokio::sync::oneshot;

use crate::cmd::agent::Agent;
use crate::cmd::balancer::Balancer;
use crate::cmd::handler::Handler as _;

#[cfg(feature = "web_admin_panel")]
pub const ESBUILD_META_CONTENTS: &str = include_str!("../esbuild-meta.json");

#[derive(Parser)]
#[command(arg_required_else_help(true), version, about, long_about = None)]
/// Stateful load balancer for llama.cpp
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[expect(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum Commands {
    /// Agent for managing llama.cpp instances
    Agent(Agent),
    /// Balances incoming requests to llama.cpp instances and optionally provides a web dashboard
    Balancer(Balancer),
}

#[actix_web::main]
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
        Some(Commands::Agent(handler)) => Ok(handler.handle(shutdown_rx).await?),
        Some(Commands::Balancer(handler)) => {
            #[cfg(feature = "web_admin_panel")]
            initialize_instance(ESBUILD_META_CONTENTS);

            Ok(handler.handle(shutdown_rx).await?)
        }
        None => Ok(()),
    }
}
