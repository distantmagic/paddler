use std::time::SystemTime;

use tokio::process::Child;

#[derive(Debug, Default)]
pub struct BalancerInstance {
    pub allowed_cors_hosts: Vec<String>,
    pub child: Option<Child>,
    pub last_update: Option<SystemTime>,
}

impl BalancerInstance {
    pub async fn cleanup(&mut self) {
        if let Some(mut balancer) = self.child.take() {
            if let Err(err) = balancer.kill().await {
                panic!("Failed to kill balancer: {err}");
            }
        }

        self.allowed_cors_hosts.clear();
    }
}
