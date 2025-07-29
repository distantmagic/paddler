use std::sync::Arc;

use log::error;
use tokio::sync::mpsc;

use crate::dispenses_slots::DispensesSlots as _;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::generated_token_result::GeneratedTokenResult;
use crate::slot_status::SlotStatus;

pub struct GenerateTokensDropGuard {
    generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
    slot_index: u32,
    slot_status: Arc<SlotStatus>,
}

impl GenerateTokensDropGuard {
    pub fn new(
        generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
        slot_index: u32,
        slot_status: Arc<SlotStatus>,
    ) -> Self {
        Self {
            generated_tokens_tx,
            slot_index,
            slot_status,
        }
    }
}

impl Drop for GenerateTokensDropGuard {
    fn drop(&mut self) {
        self.slot_status.release_slot();

        self.generated_tokens_tx
            .send(GeneratedTokenEnvelope {
                slot: self.slot_index,
                generated_token_result: GeneratedTokenResult::Done,
            })
            .unwrap_or_else(|err| {
                error!(
                    "Failed to notify about ending token generation in slot {}: {err}",
                    self.slot_index
                );
            });
    }
}
