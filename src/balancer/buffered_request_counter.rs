use std::sync::atomic::AtomicI32;
use std::sync::Arc;

use tokio::sync::Notify;

use crate::atomic_value::AtomicValue;

pub struct BufferedRequestCounter {
    count: Arc<AtomicValue<AtomicI32>>,
    pub update_notifier: Arc<Notify>,
}

impl BufferedRequestCounter {
    pub fn new(update_notifier: Arc<Notify>) -> Self {
        BufferedRequestCounter {
            count: Arc::new(AtomicValue::<AtomicI32>::new(0)),
            update_notifier,
        }
    }

    pub fn decrement(&self) {
        self.count.decrement();
        self.update_notifier.notify_waiters();
    }

    pub fn get(&self) -> i32 {
        self.count.get()
    }

    pub fn increment(&self) {
        self.count.increment();
        self.update_notifier.notify_waiters();
    }
}
