use std::time::SystemTime;

use cucumber::World;
use dashmap::DashMap;
use reqwest::Response;
use tokio::process::Child;

use crate::agents_collection::AgentsCollection;
use crate::llamacpp_instance_collection::LlamaCppInstanceCollection;
use crate::statsd_instance::StatsdInstance;

#[derive(Debug, Default, World)]
pub struct PaddlerWorld {
    pub agents: AgentsCollection,
    pub balancer: Option<Child>,
    pub buffered_request_timeout: Option<i64>,
    pub last_balancer_state_update: Option<SystemTime>,
    pub last_metrics_state_update: Option<SystemTime>,
    pub llamas: LlamaCppInstanceCollection,
    pub max_buffered_requests: Option<i64>,
    pub requests: DashMap<String, Response>,
    pub statsd: StatsdInstance,
}

impl PaddlerWorld {
    pub async fn cleanup(&mut self) {
        self.agents.cleanup().await;
        self.llamas.cleanup().await;
        self.statsd.kill().await;
        self.requests.clear();

        if let Some(mut balancer) = self.balancer.take() {
            if let Err(err) = balancer.kill().await {
                panic!("Failed to kill balancer: {err}");
            }
        }
    }
}
