use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SlotAggregatedStatusSnapshot {
    pub slots_processing: i32,
    pub slots_total: i32,
}
