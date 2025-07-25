use anyhow::Result;
use minijinja::context;
use minijinja::Environment;

use crate::agent::chat_render_params::ChatRenderParams;
use crate::agent::renders_chat_template::RendersChatTemplate;

pub struct ChatTemplate<'environment> {
    env: Environment<'environment>,
}

impl<'environment> ChatTemplate<'environment> {
    pub fn new(contents: String) -> Result<Self> {
        let mut env = Environment::new();

        env.add_template_owned("chat_template", contents)?;

        Ok(ChatTemplate { env })
    }
}

impl<'environment> RendersChatTemplate for ChatTemplate<'environment> {
    fn render(
        &self,
        ChatRenderParams {
            add_generation_prompt,
            conversation_history,
        }: ChatRenderParams,
    ) -> Result<String> {
        let template = self.env.get_template("chat_template")?;

        Ok(template.render(context! {
            add_generation_prompt => add_generation_prompt,
            messages => conversation_history,
        })?)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::conversation_message::ConversationMessage;

    #[test]
    fn test_render_template() -> Result<()> {
        let template_contents = indoc! {"
            {% for message in messages %}
                {{ message.role }}: {{ message.content }}
            {% endfor %}
        "}
        .to_string();

        let template = ChatTemplate::new(template_contents)?;

        assert_eq!(
            template.render(ChatRenderParams {
                add_generation_prompt: false,
                conversation_history: vec![
                    ConversationMessage {
                        role: "user".to_string(),
                        content: "Hi".to_string(),
                    },
                    ConversationMessage {
                        role: "assistant".to_string(),
                        content: "Ho".to_string(),
                    }
                ]
            })?,
            "\n    user: Hi\n\n    assistant: Ho\n"
        );

        Ok(())
    }
}
