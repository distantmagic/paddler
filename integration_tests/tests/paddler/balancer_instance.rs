use std::time::SystemTime;

use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use tokio::process::Child;

use crate::balancer_management_client::BalancerManagementClient;
use crate::cleanable::Cleanable;
use crate::state_database_configuration::StateDatabaseConfiguration;

#[derive(Debug, Default)]
pub struct BalancerInstance {
    pub allowed_cors_hosts: Vec<String>,
    pub buffered_request_timeout: Option<i64>,
    pub child: Option<Child>,
    pub last_update: Option<SystemTime>,
    pub management_client: BalancerManagementClient,
    pub max_buffered_requests: Option<i64>,
    pub state_database_configuration: Option<StateDatabaseConfiguration>,
    pub statsd_reporting_interval: Option<i64>,
}

#[async_trait]
impl Cleanable for BalancerInstance {
    async fn cleanup(&mut self) -> Result<()> {
        if let Some(mut balancer) = self.child.take()
            && let Err(err) = balancer.kill().await
        {
            return Err(anyhow!("Failed to kill balancer: {err}"));
        }

        self.allowed_cors_hosts.clear();

        Ok(())
    }
}
