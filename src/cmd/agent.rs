use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio::sync::oneshot;

use super::parse_socket_addr;
use crate::agent::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::agent::llamacpp_arbiter_service::LlamaCppArbiterService;
use crate::agent::management_socket_client_service::ManagementSocketClientService;
use crate::agent::reconciliation_queue::ReconciliationQueue;
use crate::agent::reconciliation_service::ReconciliationService;
use crate::service_manager::ServiceManager;

#[derive(Parser)]
pub struct Agent {
    #[arg(long, value_parser = parse_socket_addr)]
    /// Address of the llama.cpp instance that the agent will spawn and manage
    llamacpp_listen_addr: SocketAddr,

    #[arg(long, value_parser = parse_socket_addr)]
    /// Address of the management server that the agent will report to
    management_addr: SocketAddr,

    #[arg(long)]
    /// Name of the agent (optional)
    name: Option<String>,
}

impl Agent {
    pub async fn handle(&self, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        let llamacpp_applicable_state_holder = Arc::new(LlamaCppApplicableStateHolder::new());
        let reconciliation_queue = Arc::new(ReconciliationQueue::new()?);
        let mut service_manager = ServiceManager::new();

        service_manager.add_service(LlamaCppArbiterService::new(
            llamacpp_applicable_state_holder.clone(),
            self.llamacpp_listen_addr,
        )?);

        service_manager.add_service(ManagementSocketClientService::new(
            self.management_addr,
            self.name.clone(),
            reconciliation_queue.clone(),
        )?);

        service_manager.add_service(ReconciliationService::new(
            llamacpp_applicable_state_holder,
            reconciliation_queue,
        )?);

        service_manager.run_forever(shutdown_rx).await
    }
}
