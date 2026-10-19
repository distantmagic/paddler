use crate::agent_applicable_state::AgentApplicableState;
use crate::serving_startup_failure::ServingStartupFailure;
use crate::started_serving::StartedServing;

pub struct FinishedServingStartup {
    pub agent_applicable_state: AgentApplicableState,
    pub outcome: Result<StartedServing, ServingStartupFailure>,
}
