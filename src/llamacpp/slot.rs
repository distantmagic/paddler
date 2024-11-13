use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    id: i32,
    is_processing: bool,
}
