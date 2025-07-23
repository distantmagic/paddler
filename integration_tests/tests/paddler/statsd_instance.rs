use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use tokio::process::Child;

use crate::cleanable::Cleanable;
use crate::metrics::Metrics;

#[derive(Debug, Default)]
pub struct StatsdInstance {
    pub child: Option<Child>,
    pub metrics: Vec<Metrics>,
}

#[async_trait]
impl Cleanable for StatsdInstance {
    async fn cleanup(&mut self) -> Result<()> {
        if let Some(mut statsd) = self.child.take()
            && let Err(err) = statsd.kill().await
        {
            return Err(anyhow!("Failed to kill statsd: {err}"));
        }

        Ok(())
    }
}
