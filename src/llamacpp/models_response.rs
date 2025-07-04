// paddler/src/llamacpp/models_response.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    pub models: Option<Vec<Model>>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub model: String,
    // Add other fields as needed
}
