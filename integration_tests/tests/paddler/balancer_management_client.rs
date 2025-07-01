use anyhow::Result;
use anyhow::anyhow;

use crate::BALANCER_PORT;
use crate::agent_response::AgentsResponse;

#[derive(Debug, Default)]
pub struct BalancerManagementClient {}

impl BalancerManagementClient {
    pub async fn fetch_agents(&self) -> Result<AgentsResponse> {
        let response =
            reqwest::get(format!("http://127.0.0.1:{BALANCER_PORT}/api/v1/agents")).await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get agent status: {}", response.status()));
        }

        Ok(response.json::<AgentsResponse>().await?)
    }
}
