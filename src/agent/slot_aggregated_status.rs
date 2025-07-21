use tokio::sync::Notify;

use crate::agent::dispenses_slots::DispensesSlots;
use crate::atomic_value::AtomicValue;
use crate::produces_snapshot::ProducesSnapshot;
use crate::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;

pub struct SlotAggregatedStatus {
    pub slots_processing: AtomicValue,
    pub slots_total: i32,
    pub update_notifier: Notify,
}

impl SlotAggregatedStatus {
    pub fn new(slots_total: i32) -> Self {
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

impl DispensesSlots for SlotAggregatedStatus {
    fn release_slot(&self) {
        self.slots_processing.decrement();
        self.update_notifier.notify_waiters();
    }

    fn take_slot(&self) {
        self.slots_processing.increment();
        self.update_notifier.notify_waiters();
    }
}

impl ProducesSnapshot for SlotAggregatedStatus {
    type Snapshot = SlotAggregatedStatusSnapshot;

    fn make_snapshot(&self) -> Self::Snapshot {
        SlotAggregatedStatusSnapshot {
            slots_processing: self.slots_processing.get(),
            slots_total: self.slots_total,
        }
    }
}
