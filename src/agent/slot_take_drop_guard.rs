use std::sync::Arc;

use crate::agent::dispenses_slots::DispensesSlots as _;
use crate::agent::slot_metrics::SlotMetrics;

pub struct SlotTakeDropGuard {
    slot_metrics: Arc<SlotMetrics>,
}

impl SlotTakeDropGuard {
    pub fn new(slot_metrics: Arc<SlotMetrics>) -> Self {
        slot_metrics.take_slot();

        Self {
            slot_metrics,
        }
    }
}

impl Drop for SlotTakeDropGuard {
    fn drop(&mut self) {
        self.slot_metrics.release_slot();
    }
}
