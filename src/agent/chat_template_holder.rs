use std::sync::RwLock;

use anyhow::anyhow;
use anyhow::Result;

use crate::agent::chat_render_params::ChatRenderParams;
use crate::agent::chat_template::ChatTemplate;
use crate::agent::renders_chat_template::RendersChatTemplate;

pub struct ChatTemplateHolder {
    chat_template: RwLock<Option<ChatTemplate<'static>>>,
}

impl ChatTemplateHolder {
    pub fn new() -> Self {
        Self {
            chat_template: RwLock::new(None),
        }
    }

    pub fn set_chat_template(&self, chat_template: ChatTemplate<'static>) {
        let mut lock = self.chat_template.write().unwrap();

        *lock = Some(chat_template);
    }
}

impl RendersChatTemplate for ChatTemplateHolder {
    fn render(&self, params: ChatRenderParams) -> Result<String> {
        let lock = self
            .chat_template
            .read()
            .expect("Failed to acquire read lock on chat template");

        if let Some(chat_template) = lock.as_ref() {
            chat_template.render(params)
        } else {
            Err(anyhow!("Chat template is not set"))
        }
    }
}
