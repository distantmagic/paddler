use cucumber::World;
use dashmap::DashMap;
use reqwest::Response;
use tokio::process::Child;

use crate::llamacpp_instance_collection::LlamaCppInstanceCollection;

#[derive(Debug, Default, World)]
pub struct PaddlerWorld {
    pub agents: DashMap<String, Child>,
    pub balancer: Option<Child>,
    pub statsd: Option<Child>,
    pub llamas: LlamaCppInstanceCollection,
    pub requests: DashMap<String, Response>,
}

impl PaddlerWorld {
    pub async fn cleanup(&mut self) {
        self.llamas.cleanup().await;

        if let Some(mut balancer) = self.balancer.take() {
            if let Err(err) = balancer.kill().await {
                panic!("Failed to kill balancer: {err}");
            }
        }

        for mut agent in self.agents.iter_mut() {
            if let Err(err) = agent.value_mut().kill().await {
                panic!("Failed to kill agent {}: {}", agent.key(), err);
            }
        }

        if let Some(mut statsd) = self.statsd.take() {
            if let Err(err) = statsd.kill().await {
                panic!("Failed to kill statsd: {err}");
            }
        }
    }
}
