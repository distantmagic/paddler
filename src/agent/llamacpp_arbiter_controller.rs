use actix::Addr;
use tokio::sync::mpsc;

use crate::agent::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::agent::llamacpp_slot::LlamaCppSlot;

pub struct LlamaCppArbiterController {
    pub applicable_state_tx: mpsc::Sender<LlamaCppApplicableState>,
    pub llamacpp_slot_addr: Addr<LlamaCppSlot>,
}
