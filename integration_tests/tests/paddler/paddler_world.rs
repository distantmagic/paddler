use std::time::SystemTime;

use cucumber::World;
use dashmap::DashMap;
use reqwest::Response;
use tokio::process::Child;

use crate::agents_collection::AgentsCollection;
use crate::balancer_management_client::BalancerManagementClient;
use crate::llamacpp_instance_collection::LlamaCppInstanceCollection;
use crate::request_builder::RequestBuilder;
use crate::statsd_instance::StatsdInstance;

#[derive(Debug, Default, World)]
pub struct PaddlerWorld {
    pub agents: AgentsCollection,
    pub balancer: Option<Child>,
    pub balancer_allowed_cors_hosts: Vec<String>,
    pub balancer_management_client: BalancerManagementClient,
    pub buffered_request_timeout: Option<i64>,
    pub last_balancer_state_update: Option<SystemTime>,
    pub llamas: LlamaCppInstanceCollection,
    pub max_buffered_requests: Option<i64>,
    pub request_builder: RequestBuilder,
    pub responses: DashMap<String, Response>,
    pub statsd: StatsdInstance,
}

impl PaddlerWorld {
    pub async fn cleanup(&mut self) {
        self.agents.cleanup().await;
        self.balancer_allowed_cors_hosts.clear();
        self.llamas.cleanup().await;
        self.request_builder.cleanup();
        self.statsd.kill().await;
        self.responses.clear();

        if let Some(mut balancer) = self.balancer.take() {
            if let Err(err) = balancer.kill().await {
                panic!("Failed to kill balancer: {err}");
            }
        }
    }
}
