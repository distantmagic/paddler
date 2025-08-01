use serde::Serialize;

use crate::chat_template_head::ChatTemplateHead;

#[derive(Serialize)]
pub struct ChatTemplateHeads {
    pub chat_template_heads: Vec<ChatTemplateHead>,
}
