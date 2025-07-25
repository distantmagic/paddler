use anyhow::Result;

use crate::agent::chat_render_params::ChatRenderParams;

pub trait RendersChatTemplate {
    fn render(&self, chat_render_params: ChatRenderParams) -> Result<String>;
}
