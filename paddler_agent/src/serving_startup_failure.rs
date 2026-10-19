use crate::agent_error::AgentError;
use crate::serving_startup_plan::ServingStartupPlan;

pub struct ServingStartupFailure {
    pub error: AgentError,
    pub retry_plan: ServingStartupPlan,
}
