use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatTemplateHead {
    pub id: String,
    pub name: String,
}
