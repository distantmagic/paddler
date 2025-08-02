use serde::Serialize;

use crate::chat_template::ChatTemplate;

#[derive(Serialize)]
pub struct ChatTemplateOverride {
    pub chat_template_override: Option<ChatTemplate>,
}
