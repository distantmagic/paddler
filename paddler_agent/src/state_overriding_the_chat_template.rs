use paddler_messaging::chat_template::ChatTemplate;
use paddler_messaging::chat_template_source::ChatTemplateSource;

use crate::agent_applicable_state::AgentApplicableState;

pub fn state_overriding_the_chat_template() -> AgentApplicableState {
    AgentApplicableState::WithoutModel {
        chat_template_source: ChatTemplateSource::Override(ChatTemplate {
            content: "{{ messages }}".to_owned(),
        }),
    }
}
