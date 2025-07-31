use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AgentIssue {
    HuggingFaceCannotAcquireLock(String),
    HuggingFaceModelDoesNotExist(String),
    ModelCannotBeLoaded(String),
    ModelFileDoesNotExist(String),
    UnableToFindChatTemplate(String),
}
