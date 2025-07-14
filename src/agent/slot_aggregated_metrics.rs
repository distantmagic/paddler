use tokio::sync::Notify;

use crate::agent::dispenses_slots::DispensesSlots;
use crate::atomic_value::AtomicValue;

pub struct SlotAggregatedMetrics {
    pub slots_processing: AtomicValue,
    pub slots_total: usize,
    pub update_notifier: Notify,
}

impl SlotAggregatedMetrics {
    pub fn new(slots_total: usize) -> Self {
        Self {
            slots_processing: AtomicValue::new(0),
            slots_total,
            update_notifier: Notify::new(),
        }
    }

    pub fn reset(&self) {
        self.slots_processing.reset();
        self.update_notifier.notify_waiters();
    }
}

impl DispensesSlots for SlotAggregatedMetrics {
    fn release_slot(&self) {
        self.slots_processing.decrement();
        self.update_notifier.notify_waiters();
    }

    fn take_slot(&self) {
        self.slots_processing.increment();
        self.update_notifier.notify_waiters();
    }
}
