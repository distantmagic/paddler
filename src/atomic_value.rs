use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

pub struct AtomicValue<TAtomic> {
    value: TAtomic,
}

impl AtomicValue<AtomicI32> {
    pub fn new(initial: i32) -> Self {
        Self {
            value: AtomicI32::new(initial),
        }
    }

    pub fn decrement(&self) {
        self.value.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn get(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::SeqCst);
    }

    pub fn reset(&self) {
        self.value.store(0, Ordering::SeqCst);
    }

    pub fn set(&self, value: i32) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn set_check(&self, value: i32) -> bool {
        if self.get() != value {
            self.set(value);

            true
        } else {
            false
        }
    }
}

impl AtomicValue<AtomicU64> {
    pub fn new(initial: u64) -> Self {
        Self {
            value: AtomicU64::new(initial),
        }
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::SeqCst);
    }
}
