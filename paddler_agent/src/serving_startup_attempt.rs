use crate::agent_applicable_state::AgentApplicableState;
use crate::retry_delay::RetryDelay;
use crate::serving_startup_plan::ServingStartupPlan;

pub struct ServingStartupAttempt {
    pub agent_applicable_state: AgentApplicableState,
    pub retry_delay: RetryDelay,
    pub serving_startup_plan: ServingStartupPlan,
}
