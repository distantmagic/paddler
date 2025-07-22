use std::sync::Arc;

use crate::agent::dispenses_slots::DispensesSlots as _;
use crate::agent::slot_status::SlotStatus;

pub struct GenerateTokensDropGuard {
    slot_status: Arc<SlotStatus>,
}

impl GenerateTokensDropGuard {
    pub fn new(slot_status: Arc<SlotStatus>) -> Self {
        slot_status.take_slot();

        Self { slot_status }
    }
}

impl Drop for GenerateTokensDropGuard {
    fn drop(&mut self) {
        self.slot_status.release_slot();
    }
}
