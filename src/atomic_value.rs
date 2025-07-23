use std::sync::atomic::AtomicI32;

pub struct AtomicValue {
    value: AtomicI32,
}

impl AtomicValue {
    pub fn new(initial: i32) -> Self {
        AtomicValue {
            value: AtomicI32::new(initial),
        }
    }

    pub fn decrement(&self) {
        self.value.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get(&self) -> i32 {
        self.value.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set(&self, value: i32) {
        self.value.store(value, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn reset(&self) {
        self.value.store(0, std::sync::atomic::Ordering::SeqCst);
    }
}
