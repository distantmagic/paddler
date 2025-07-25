use std::sync::atomic::AtomicI32;
use std::sync::Arc;

use crate::atomic_value::AtomicValue;

pub struct BufferedRequestCountGuard {
    buffered_requests_count: Arc<AtomicValue<AtomicI32>>,
}

impl BufferedRequestCountGuard {
    pub fn new(buffered_requests_count: Arc<AtomicValue<AtomicI32>>) -> Self {
        BufferedRequestCountGuard {
            buffered_requests_count,
        }
    }
}

impl Drop for BufferedRequestCountGuard {
    fn drop(&mut self) {
        self.buffered_requests_count.decrement();
    }
}
