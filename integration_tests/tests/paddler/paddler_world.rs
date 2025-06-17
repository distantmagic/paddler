use cucumber::World;
use dashmap::DashMap;
use reqwest::Response;
use tokio::process::Child;

use crate::agents_collection::AgentsCollection;
use crate::llamacpp_instance_collection::LlamaCppInstanceCollection;

#[derive(Debug, Default, World)]
pub struct PaddlerWorld {
    pub agents: AgentsCollection,
    pub balancer: Option<Child>,
    pub statsd: Option<Child>,
    pub llamas: LlamaCppInstanceCollection,
    pub requests: DashMap<String, Response>,
}

impl PaddlerWorld {
    pub async fn cleanup(&mut self) {
        self.agents.cleanup().await;
        self.llamas.cleanup().await;
        self.requests.clear();

        if let Some(mut balancer) = self.balancer.take() {
            if let Err(err) = balancer.kill().await {
                panic!("Failed to kill balancer: {err}");
            }
        }

        if let Some(mut statsd) = self.statsd.take() {
            if let Err(err) = statsd.kill().await {
                panic!("Failed to kill statsd: {err}");
            }
        }
    }
}
