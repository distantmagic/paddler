use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    id: usize,
    is_processing: bool,
}
