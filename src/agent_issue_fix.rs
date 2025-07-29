use crate::agent_issue::AgentIssue;

pub enum AgentIssueFix {
    HuggingFaceDownloadedModel,
    HuggingFaceStartedDownloading,
    ModelStateIsReconciled,
    ModelFileExists,
    ModelIsLoaded,
}

impl AgentIssueFix {
    pub fn can_fix(&self, issue: &AgentIssue) -> bool {
        match issue {
            AgentIssue::HuggingFaceCannotAcquireLock(_) => matches!(
                self,
                AgentIssueFix::HuggingFaceDownloadedModel
                    | AgentIssueFix::HuggingFaceStartedDownloading
                    | AgentIssueFix::ModelStateIsReconciled
            ),
            AgentIssue::HuggingFaceModelDoesNotExist(_) => matches!(
                self,
                AgentIssueFix::HuggingFaceDownloadedModel
                    | AgentIssueFix::HuggingFaceStartedDownloading
                    | AgentIssueFix::ModelStateIsReconciled
            ),
            AgentIssue::ModelCannotBeLoaded(_) => matches!(self, AgentIssueFix::ModelIsLoaded),
            AgentIssue::ModelFileDoesNotExist(_) => matches!(self, AgentIssueFix::ModelFileExists),
        }
    }
}
