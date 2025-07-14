use std::sync::Arc;

use crate::agent::dispenses_slots::DispensesSlots;
use crate::agent::slot_aggregated_metrics::SlotAggregatedMetrics;
use crate::atomic_value::AtomicValue;

pub struct SlotMetrics {
    pub slot_aggregated_metrics: Arc<SlotAggregatedMetrics>,
    pub slots_processing: Arc<AtomicValue>,
}

impl SlotMetrics {
    pub fn new(slot_aggregated_metrics: Arc<SlotAggregatedMetrics>) -> Self {
        Self {
            slot_aggregated_metrics,
            slots_processing: Arc::new(AtomicValue::new(0)),
        }
    }
}

impl DispensesSlots for SlotMetrics {
    fn release_slot(&self) {
        self.slots_processing.decrement();
        self.slot_aggregated_metrics.release_slot();
    }

    fn take_slot(&self) {
        self.slots_processing.increment();
        self.slot_aggregated_metrics.take_slot();
    }
}
