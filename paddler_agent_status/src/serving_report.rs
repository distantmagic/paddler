use std::collections::BTreeSet;

use paddler_messaging::agent_issue::AgentIssue;
use paddler_messaging::inference_mode::InferenceMode;

pub struct ServingReport {
    pub inference_mode: InferenceMode,
    pub model_path: String,
    pub serving_issues: BTreeSet<AgentIssue>,
    pub uses_chat_template_override: bool,
}
