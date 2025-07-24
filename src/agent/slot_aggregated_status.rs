use std::sync::atomic::AtomicI32;
use std::sync::RwLock;

use tokio::sync::Notify;

use crate::agent::dispenses_slots::DispensesSlots;
use crate::atomic_value::AtomicValue;
use crate::produces_snapshot::ProducesSnapshot;
use crate::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;

pub struct SlotAggregatedStatus {
    pub desired_slots_total: i32,
    pub model_path: RwLock<Option<String>>,
    pub slots_processing: AtomicValue<AtomicI32>,
    pub slots_total: AtomicValue<AtomicI32>,
    pub update_notifier: Notify,
    pub version: AtomicValue<AtomicI32>,
}

impl SlotAggregatedStatus {
    pub fn new(desired_slots_total: i32) -> Self {
        Self {
            desired_slots_total,
            model_path: RwLock::new(None),
            slots_processing: AtomicValue::<AtomicI32>::new(0),
            slots_total: AtomicValue::<AtomicI32>::new(0),
            update_notifier: Notify::new(),
            version: AtomicValue::<AtomicI32>::new(0),
        }
    }

    pub fn reset(&self) {
        self.set_model_path(None);
        self.slots_processing.reset();
        self.slots_total.reset();
        self.version.increment();
        self.update_notifier.notify_waiters();
    }

    pub fn set_model_path(&self, model_path: Option<String>) {
        let mut path_lock = self.model_path.write().unwrap_or_else(|err| {
            panic!("Lock poisoned when setting model path: {model_path:?}, error: {err:?}")
        });

        *path_lock = model_path;

        self.version.increment();
        self.update_notifier.notify_waiters();
    }
}

impl DispensesSlots for SlotAggregatedStatus {
    fn release_slot(&self) {
        self.slots_processing.decrement();
        self.version.increment();

        self.update_notifier.notify_waiters();
    }

    fn take_slot(&self) {
        self.slots_processing.increment();
        self.version.increment();

        self.update_notifier.notify_waiters();
    }
}

impl ProducesSnapshot for SlotAggregatedStatus {
    type Snapshot = SlotAggregatedStatusSnapshot;

    fn make_snapshot(&self) -> Self::Snapshot {
        SlotAggregatedStatusSnapshot {
            desired_slots_total: self.desired_slots_total,
            model_path: self
                .model_path
                .read()
                .expect("Lock poisoned when getting model path")
                .clone(),
            slots_processing: self.slots_processing.get(),
            slots_total: self.slots_total.get(),
            version: self.version.get(),
        }
    }
}
