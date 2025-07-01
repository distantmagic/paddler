use std::time::SystemTime;

use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use cucumber::World;
use dashmap::DashMap;
use reqwest::Response;
use tokio::process::Child;

use crate::agent_collection::AgentCollection;
use crate::balancer_management_client::BalancerManagementClient;
use crate::cleanable::Cleanable;
use crate::llamacpp_instance_collection::LlamaCppInstanceCollection;
use crate::request_builder::RequestBuilder;
use crate::supervisor_collection::SupervisorCollection;

#[derive(Debug, Default, World)]
pub struct PaddlerWorld {
    pub agents: AgentCollection,
    pub balancer: Option<Child>,
    pub balancer_allowed_cors_hosts: Vec<String>,
    pub balancer_management_client: BalancerManagementClient,
    pub buffered_request_timeout: Option<i64>,
    pub last_balancer_state_update: Option<SystemTime>,
    pub llamas: LlamaCppInstanceCollection,
    pub max_buffered_requests: Option<i64>,
    pub request_builder: RequestBuilder,
    pub responses: DashMap<String, Response>,
    pub statsd: Option<Child>,
    pub supervisors: SupervisorCollection,
}

#[async_trait]
impl Cleanable for PaddlerWorld {
    async fn cleanup(&mut self) -> Result<()> {
        self.agents.cleanup().await?;
        self.balancer_allowed_cors_hosts.clear();
        self.llamas.cleanup().await?;
        self.request_builder.cleanup().await?;
        self.responses.clear();
        self.supervisors.cleanup().await?;

        if let Some(mut balancer) = self.balancer.take()
            && let Err(err) = balancer.kill().await
        {
            return Err(anyhow!("Failed to kill balancer: {err}"));
        }

        if let Some(mut statsd) = self.statsd.take()
            && let Err(err) = statsd.kill().await
        {
            return Err(anyhow!("Failed to kill statsd: {err}"));
        }

        Ok(())
    }
}
