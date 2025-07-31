use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::time::timeout;

use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::buffered_request_agent_wait_result::BufferedRequestAgentWaitResult;
use crate::balancer::buffered_request_count_guard::BufferedRequestCountGuard;

pub struct BufferedRequestManager {
    agent_controller_pool: Arc<AgentControllerPool>,
    buffered_request_timeout: Duration,
    pub buffered_requests_count: Arc<AtomicValue<AtomicI32>>,
    max_buffered_requests: i32,
}

impl BufferedRequestManager {
    pub fn new(
        agent_controller_pool: Arc<AgentControllerPool>,
        buffered_request_timeout: Duration,
        max_buffered_requests: i32,
    ) -> Self {
        Self {
            agent_controller_pool,
            buffered_request_timeout,
            buffered_requests_count: Arc::new(AtomicValue::<AtomicI32>::new(0)),
            max_buffered_requests,
        }
    }

    pub async fn wait_for_available_agent(&self) -> Result<BufferedRequestAgentWaitResult> {
        if self.buffered_requests_count.get() >= self.max_buffered_requests {
            return Ok(BufferedRequestAgentWaitResult::BufferOverflow);
        }

        // Do a quick check before getting into the coroutines
        if let Some(agent_controller) = self.agent_controller_pool.find_least_busy_agent_controller() {
            return Ok(BufferedRequestAgentWaitResult::Found(
                agent_controller,
            ));
        }

        self.buffered_requests_count.increment();

        let _buffered_request_count_guard = BufferedRequestCountGuard::new(self.buffered_requests_count.clone());
        let agent_controller_pool = self.agent_controller_pool.clone();

        match timeout(self.buffered_request_timeout, async {
            loop {
                match agent_controller_pool.find_least_busy_agent_controller() {
                    Some(agent_controller) => {
                        return Ok::<_, anyhow::Error>(BufferedRequestAgentWaitResult::Found(
                            agent_controller,
                        ))
                    }
                    None => agent_controller_pool.update_notifier.notified().await,
                }
            }
        })
        .await
        {
            Ok(inner_result) => Ok(inner_result?),
            Err(timeout_err) => Ok(BufferedRequestAgentWaitResult::Timeout(timeout_err.into())),
        }
    }
}
