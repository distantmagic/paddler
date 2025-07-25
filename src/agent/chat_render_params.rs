use crate::conversation_message::ConversationMessage;

pub struct ChatRenderParams {
    pub add_generation_prompt: bool,
    pub conversation_history: Vec<ConversationMessage>,
}
