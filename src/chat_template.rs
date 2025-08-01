use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;

use crate::chat_template_head::ChatTemplateHead;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatTemplate {
    pub content: String,
    pub id: String,
    pub name: String,
    pub updated_at: DateTime<Utc>,
}

impl ChatTemplate {
    pub fn to_head(&self) -> ChatTemplateHead {
        ChatTemplateHead {
            id: self.id.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
        }
    }
}
