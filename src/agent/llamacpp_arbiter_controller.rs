use actix::Addr;
use tokio::sync::oneshot;

use crate::agent::llamacpp_slot::LlamaCppSlot;

pub struct LlamaCppArbiterController {
    pub llamacpp_slot_addr: Addr<LlamaCppSlot>,
    pub shutdown_tx: oneshot::Sender<()>,
}
