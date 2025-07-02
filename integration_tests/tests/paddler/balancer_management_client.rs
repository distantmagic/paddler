use anyhow::Result;
use anyhow::anyhow;

use crate::BALANCER_PORT;
use crate::agent_response::AgentsResponse;
use crate::supervisor_response::SupervisorsResponse;

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

    pub async fn fetch_supervisors(&self) -> Result<SupervisorsResponse> {
        let response = reqwest::get(format!(
            "http://127.0.0.1:{BALANCER_PORT}/api/v1/supervisors"
        ))
        .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to get supervisor status: {}",
                response.status()
            ));
        }

        Ok(response.json::<SupervisorsResponse>().await?)
    }
}
