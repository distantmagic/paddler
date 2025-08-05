use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub enum GeneratedTokenResult {
    ChatTemplateError(String),
    Done,
    Token(String),
}
