use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize)]
pub struct ModelMetadata {
    pub metadata: BTreeMap<String, String>,
}

impl ModelMetadata {
    pub fn new() -> Self {
        ModelMetadata {
            metadata: BTreeMap::new(),
        }
    }

    pub fn set_meta_field(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}
