use std::sync::Arc;

use crate::agent::dispenses_slots::DispensesSlots;
use crate::agent::slot_aggregated_status::SlotAggregatedStatus;
use crate::atomic_value::AtomicValue;

pub struct SlotStatus {
    // pub model_path: Option<String>,
    pub slot_aggregated_status: Arc<SlotAggregatedStatus>,
    pub slots_processing: Arc<AtomicValue>,
}

impl SlotStatus {
    pub fn new(slot_aggregated_status: Arc<SlotAggregatedStatus>) -> Self {
        Self {
            slot_aggregated_status,
            slots_processing: Arc::new(AtomicValue::new(0)),
        }
    }

    pub fn started(&self) {
        self.slot_aggregated_status.slots_total.increment();
        self.slot_aggregated_status.update_notifier.notify_waiters();
    }

    pub fn stopped(&self) {
        self.slot_aggregated_status.slots_total.decrement();
        self.slot_aggregated_status.update_notifier.notify_waiters();
    }
}

impl DispensesSlots for SlotStatus {
    fn release_slot(&self) {
        self.slots_processing.decrement();
        self.slot_aggregated_status.release_slot();
    }

    fn take_slot(&self) {
        self.slots_processing.increment();
        self.slot_aggregated_status.take_slot();
    }
}
