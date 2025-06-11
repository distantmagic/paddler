use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    pub id: usize,
    pub is_processing: bool,
}
