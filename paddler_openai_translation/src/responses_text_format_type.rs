use serde::Deserialize;

#[derive(Deserialize)]
pub enum ResponsesTextFormatType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json_schema")]
    JsonSchema,
}
