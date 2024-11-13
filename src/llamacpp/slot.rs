use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    id: i32,
    is_processing: bool,
}
