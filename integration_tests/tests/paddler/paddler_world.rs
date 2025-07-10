use anyhow::Result;
use async_trait::async_trait;
use cucumber::World;
use dashmap::DashMap;
use reqwest::Response;

use crate::agent_instance_collection::AgentInstanceCollection;
use crate::balancer_instance::BalancerInstance;
use crate::cleanable::Cleanable;
use crate::request_builder::RequestBuilder;
use crate::statsd_instance::StatsdInstance;

#[derive(Debug, Default, World)]
pub struct PaddlerWorld {
    pub agents: AgentInstanceCollection,
    pub balancer: BalancerInstance,
    pub request_builder: RequestBuilder,
    pub responses: DashMap<String, Response>,
    pub statsd: StatsdInstance,
}

#[async_trait]
impl Cleanable for PaddlerWorld {
    async fn cleanup(&mut self) -> Result<()> {
        self.agents.cleanup().await?;
        self.balancer.cleanup().await?;
        self.request_builder.cleanup().await?;
        self.responses.clear();
        self.statsd.cleanup().await?;

        Ok(())
    }
}
