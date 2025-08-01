use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicUsize;
use std::sync::RwLock;

use dashmap::DashSet;
use tokio::sync::Notify;

use crate::agent_issue::AgentIssue;
use crate::agent_issue_fix::AgentIssueFix;
use crate::atomic_value::AtomicValue;
use crate::dispenses_slots::DispensesSlots;
use crate::produces_snapshot::ProducesSnapshot;
use crate::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;

pub struct SlotAggregatedStatus {
    desired_slots_total: i32,
    download_current: AtomicValue<AtomicUsize>,
    download_filename: RwLock<Option<String>>,
    download_total: AtomicValue<AtomicUsize>,
    is_state_applied: AtomicValue<AtomicBool>,
    issues: DashSet<AgentIssue>,
    model_path: RwLock<Option<String>>,
    slots_processing: AtomicValue<AtomicI32>,
    slots_total: AtomicValue<AtomicI32>,
    pub update_notifier: Notify,
    version: AtomicValue<AtomicI32>,
}

impl SlotAggregatedStatus {
    pub fn new(desired_slots_total: i32) -> Self {
        Self {
            desired_slots_total,
            download_current: AtomicValue::<AtomicUsize>::new(0),
            download_filename: RwLock::new(None),
            download_total: AtomicValue::<AtomicUsize>::new(0),
            is_state_applied: AtomicValue::<AtomicBool>::new(true),
            issues: DashSet::new(),
            model_path: RwLock::new(None),
            slots_processing: AtomicValue::<AtomicI32>::new(0),
            slots_total: AtomicValue::<AtomicI32>::new(0),
            update_notifier: Notify::new(),
            version: AtomicValue::<AtomicI32>::new(0),
        }
    }

    pub fn decrement_total_slots(&self) {
        self.slots_total.decrement();
        self.version.increment();
        self.update_notifier.notify_waiters();
    }

    pub fn get_is_state_applied(&self) -> bool {
        self.is_state_applied.get()
    }

    pub fn has_issue(&self, issue: &AgentIssue) -> bool {
        self.issues.contains(issue)
    }

    pub fn increment_download_current(&self, size: usize) {
        self.download_current.increment_by(size);
        self.version.increment();
        self.update_notifier.notify_waiters();
    }

    pub fn increment_total_slots(&self) {
        self.slots_total.increment();
        self.version.increment();
        self.update_notifier.notify_waiters();
    }

    pub fn register_issue(&self, issue: AgentIssue) {
        if self.issues.insert(issue) {
            self.update_notifier.notify_waiters();
        }
    }

    pub fn register_fix(&self, fix: AgentIssueFix) {
        let size_before = self.issues.len();

        self.issues.retain(|issue| !fix.can_fix(issue));

        if self.issues.len() < size_before {
            self.update_notifier.notify_waiters();
        }
    }

    pub fn reset(&self) {
        self.set_model_path(None);
        self.slots_processing.reset();
        self.slots_total.reset();
        self.version.increment();
        self.update_notifier.notify_waiters();
    }

    pub fn reset_download(&self) {
        self.download_current.set(0);
        self.download_total.set(0);
        self.set_download_filename(None);
        self.version.increment();
        self.update_notifier.notify_waiters();
    }

    pub fn set_download_status(&self, current: usize, total: usize, filename: Option<String>) {
        self.download_current.set(current);
        self.download_total.set(total);
        self.set_download_filename(filename);
    }

    pub fn set_download_filename(&self, filename: Option<String>) {
        let mut filename_lock = self.download_filename.write().unwrap_or_else(|err| {
            panic!("Lock poisoned when setting download filename: {filename:?}, error: {err:?}")
        });

        *filename_lock = filename;

        self.version.increment();

        self.update_notifier.notify_waiters();
    }

    pub fn set_is_state_applied(&self, is_applied: bool) {
        self.is_state_applied.set(is_applied);
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
            issues: self.issues.iter().map(|item| item.clone()).collect(),
            desired_slots_total: self.desired_slots_total,
            download_current: self.download_current.get(),
            download_filename: self
                .download_filename
                .read()
                .expect("Lock poisoned when getting download filename")
                .clone(),
            download_total: self.download_total.get(),
            is_state_applied: self.is_state_applied.get(),
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
