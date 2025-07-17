use std::sync::Arc;
use std::thread;

use actix::sync::SyncArbiter;
use actix::System;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use tokio::sync::oneshot;

use crate::agent::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::agent::llamacpp_arbiter_controller::LlamaCppArbiterController;
use crate::agent::llamacpp_slot::LlamaCppSlot;
use crate::agent::slot_aggregated_metrics_manager::SlotAggregatedMetricsManager;

pub struct LlamaCppArbiter {
    applicable_state: LlamaCppApplicableState,
    slot_aggregated_metrics_manager: Arc<SlotAggregatedMetricsManager>,
    slots_total: i32,
}

impl LlamaCppArbiter {
    pub fn new(
        applicable_state: LlamaCppApplicableState,
        slot_aggregated_metrics_manager: Arc<SlotAggregatedMetricsManager>,
        slots_total: i32,
    ) -> Self {
        Self {
            applicable_state,
            slot_aggregated_metrics_manager,
            slots_total,
        }
    }

    pub async fn spawn(&self) -> Result<LlamaCppArbiterController> {
        let (llamacpp_slot_addr_tx, llamacpp_slot_addr_rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let model_path = self.applicable_state.model_path.clone();
        let slot_aggregated_metrics_manager = self.slot_aggregated_metrics_manager.clone();
        let slots_total = self.slots_total;

        let sync_arbiter_thread_handle = thread::spawn(move || -> Result<()> {
            let backend = Arc::new(LlamaBackend::init()?);
            let ctx_params = Arc::new(LlamaContextParams::default());
            let backend_clone = backend.clone();
            let model = Arc::new(LlamaModel::load_from_file(
                &backend_clone.clone(),
                model_path,
                &LlamaModelParams::default(),
            )?);

            let system = System::new();

            system.block_on(async move {
                llamacpp_slot_addr_tx
                    .send(SyncArbiter::start(slots_total as usize, move || {
                        LlamaCppSlot::new(
                            backend.clone(),
                            ctx_params.clone(),
                            model.clone(),
                            slot_aggregated_metrics_manager.bind_slot_metrics(),
                        )
                        .expect("Failed to create LlamaCppSlot")
                    }))
                    .expect("Failed to send LlamaCppSlot address");

                shutdown_rx
                    .await
                    .expect("Failed to receive shutdown signal");

                System::current().stop();
            });

            Ok(())
        });

        Ok(LlamaCppArbiterController::new(
            llamacpp_slot_addr_rx.await?,
            shutdown_tx,
            sync_arbiter_thread_handle,
        ))
    }
}

#[cfg(test)]
#[cfg(feature = "tests_that_use_llms")]
mod tests {
    use futures::future::join_all;
    use tokio::sync::mpsc;

    use super::*;
    use crate::agent::converts_to_applicable_state::ConvertsToApplicableState as _;
    use crate::agent::huggingface_model_reference::HuggingFaceModelReference;
    use crate::agent::llamacpp_desired_model::LlamaCppDesiredModel;
    use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;
    use crate::agent::message::GenerateTokens;

    const SLOTS_TOTAL: usize = 3;

    #[actix_web::test]
    async fn test_llamacpp_arbiter_spawn() -> Result<()> {
        let desired_state = LlamaCppDesiredState {
            model: LlamaCppDesiredModel::HuggingFace(HuggingFaceModelReference {
                filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
                repo: "Qwen/Qwen3-0.6B-GGUF".to_string(),
                // filename: "Qwen3-8B-Q4_K_M.gguf".to_string(),
                // repo: "Qwen/Qwen3-8B-GGUF".to_string(),
            }),
        };
        let slot_aggregated_metrics_manager =
            Arc::new(SlotAggregatedMetricsManager::new(SLOTS_TOTAL));

        let applicable_state = desired_state
            .to_applicable_state()
            .await?
            .expect("Failed to convert to applicable state");

        let llamacpp_arbiter = LlamaCppArbiter::new(
            applicable_state,
            slot_aggregated_metrics_manager,
            SLOTS_TOTAL,
        );
        let controller = llamacpp_arbiter.spawn().await?;

        let prompt =
            "<|im_start|>user\nHow can I make a cat happy?<|im_end|>\n<|im_start|>assistant\n";
        let (tx, mut rx) = mpsc::channel(100);

        let futures = vec![
            controller.llamacpp_slot_addr.send(GenerateTokens {
                chunk_sender: tx.clone(),
                max_tokens: 100,
                prompt: prompt.to_string(),
            }),
            controller.llamacpp_slot_addr.send(GenerateTokens {
                chunk_sender: tx.clone(),
                max_tokens: 100,
                prompt: prompt.to_string(),
            }),
            controller.llamacpp_slot_addr.send(GenerateTokens {
                chunk_sender: tx,
                max_tokens: 100,
                prompt: prompt.to_string(),
            }),
        ];

        tokio::spawn(async move {
            while let Some(chunk) = rx.recv().await {
                println!("Received chunk: {chunk}");
            }
        });

        let results = join_all(futures).await;

        for result in results {
            if let Err(err) = result {
                eprintln!("Error generating response: {err}");
            }
        }

        controller.shutdown().await?;

        Ok(())
    }
}
