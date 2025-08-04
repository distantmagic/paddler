use anyhow::Result;
use minijinja::Environment;
use minijinja::Error;
use minijinja::ErrorKind;
use serde::ser::Serialize;

use crate::chat_template::ChatTemplate;

const CHAT_TEMPLATE_NAME: &str = "chat_template";

// Known uses:
// https://huggingface.co/bartowski/Mistral-7B-Instruct-v0.3-GGUF
fn minijinja_raise_exception(message: String) -> std::result::Result<String, Error> {
    Err(Error::new::<String>(
        ErrorKind::InvalidOperation,
        format!("Model's chat template raised an exception: '{message}'"),
    ))
}

pub struct ChatTemplateRenderer {
    minijinja_env: Environment<'static>,
}

impl ChatTemplateRenderer {
    pub fn new(ChatTemplate {
        content,
    }: ChatTemplate) -> Result<Self> {
        let mut minijinja_env = Environment::new();

        minijinja_env.add_function("raise_exception", minijinja_raise_exception);
        minijinja_env.add_template_owned(CHAT_TEMPLATE_NAME, content)?;

        Ok(Self {
            minijinja_env,
        })
    }

    pub fn render<TContext: Serialize>(&self, context: TContext) -> Result<String> {
        Ok(self
            .minijinja_env
            .get_template(CHAT_TEMPLATE_NAME)?
            .render(context)?)
    }
}
