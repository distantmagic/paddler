use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    pub id: usize,
    pub is_processing: bool,
}
