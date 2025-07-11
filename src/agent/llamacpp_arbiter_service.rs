use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use log::error;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::agent::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::agent::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::agent::message::GenerateTokens;
use crate::service::Service;

pub struct LlamaCppArbiterService {
    llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
    generate_tokens_rx: mpsc::Receiver<GenerateTokens>,
    slots: usize,
}

impl LlamaCppArbiterService {
    pub async fn new(
        llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
        slots: usize,
    ) -> Result<Self> {
        let (_, generate_tokens_rx) = mpsc::channel(100);

        Ok(LlamaCppArbiterService {
            llamacpp_applicable_state_holder,
            generate_tokens_rx,
            slots,
        })
    }

    async fn on_reconciled_state_change(
        &mut self,
        llamacpp_applicable_state: Option<LlamaCppApplicableState>,
    ) -> Result<()> {
        // if let Some(llamacpp_slot_arbiter_addr) = &self.llamacpp_slot_arbiter_addr {
        // }

        Ok(())
    }
}

#[async_trait]
impl Service for LlamaCppArbiterService {
    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut reconciled_state = self.llamacpp_applicable_state_holder.subscribe();

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    debug!("Shutting down monitoring service");

                    return Ok(());
                },
                _ = reconciled_state.changed() => {
                    let llamacpp_applicable_state: Option<LlamaCppApplicableState> = reconciled_state.borrow_and_update().clone();

                    if let Err(err) = self.on_reconciled_state_change(llamacpp_applicable_state).await {
                        error!("Failed to apply reconciled state change: {err}");
                    }
                }
            }
        }
    }
}

// #[cfg(test)]
// #[cfg(feature = "tests_that_use_llms")]
// mod tests {
//     use super::*;
//     use crate::agent::converts_to_applicable_state::ConvertsToApplicableState as _;
//     use crate::agent::huggingface_model_reference::HuggingFaceModelReference;
//     use crate::agent::llamacpp_desired_model::LlamaCppDesiredModel;
//     use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;
//     use crate::agent::message::GenerateTokens;
//
//     #[actix_web::test]
//     async fn test_llamacpp_arbiter_service_run() -> Result<()> {
//         let llamacpp_applicable_state_holder = Arc::new(LlamaCppApplicableStateHolder::new());
//         let mut service = LlamaCppArbiterService::new(llamacpp_applicable_state_holder, 2).await?;
//
//         let desired_state = LlamaCppDesiredState {
//             model: LlamaCppDesiredModel::HuggingFace(HuggingFaceModelReference {
//                 filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
//                 repo: "Qwen/Qwen3-0.6B-GGUF".to_string(),
//             }),
//         };
//
//         service
//             .on_reconciled_state_change(desired_state.to_applicable_state().await?)
//             .await?;
//
//         Err(anyhow::anyhow!("This test is not fully implemented yet."))
//         // Ok(())
//     }
// }
