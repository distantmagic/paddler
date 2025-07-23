use std::sync::Arc;

use crate::atomic_value::AtomicValue;

pub struct BufferedRequestCountGuard {
    buffered_requests_count: Arc<AtomicValue>,
}

impl BufferedRequestCountGuard {
    pub fn new(buffered_requests_count: Arc<AtomicValue>) -> Self {
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
