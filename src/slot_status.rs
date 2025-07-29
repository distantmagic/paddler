use std::sync::atomic::AtomicI32;
use std::sync::Arc;

use crate::atomic_value::AtomicValue;
use crate::dispenses_slots::DispensesSlots;
use crate::slot_aggregated_status::SlotAggregatedStatus;

pub struct SlotStatus {
    pub slot_aggregated_status: Arc<SlotAggregatedStatus>,
    pub slots_processing: Arc<AtomicValue<AtomicI32>>,
}

impl SlotStatus {
    pub fn new(slot_aggregated_status: Arc<SlotAggregatedStatus>) -> Self {
        Self {
            slot_aggregated_status,
            slots_processing: Arc::new(AtomicValue::<AtomicI32>::new(0)),
        }
    }

    pub fn started(&self) {
        self.slot_aggregated_status.increment_total_slots();
    }

    pub fn stopped(&self) {
        self.slot_aggregated_status.decrement_total_slots();
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
