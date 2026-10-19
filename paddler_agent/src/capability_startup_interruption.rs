use crate::agent_error::AgentError;

#[derive(Debug)]
pub enum CapabilityStartupInterruption {
    Cancelled,
    Failed(AgentError),
}

impl From<AgentError> for CapabilityStartupInterruption {
    fn from(agent_error: AgentError) -> Self {
        Self::Failed(agent_error)
    }
}
