use serde::Serialize;

use crate::chat_template::ChatTemplate as ChatTemplateModel;

#[derive(Serialize)]
pub struct ChatTemplate {
    pub chat_template: ChatTemplateModel,
}
