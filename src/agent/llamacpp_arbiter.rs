use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use actix::sync::SyncArbiter;
use actix::System;
use anyhow::Context as _;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use tokio::sync::oneshot;

use crate::agent::llamacpp_arbiter_controller::LlamaCppArbiterController;
use crate::agent::llamacpp_slot::LlamaCppSlot;
use crate::agent::slot_aggregated_status_manager::SlotAggregatedStatusManager;
use crate::agent_applicable_state::AgentApplicableState;

pub struct LlamaCppArbiter {
    agent_name: Option<String>,
    applicable_state: AgentApplicableState,
    desired_slots_total: i32,
    slot_aggregated_status_manager: Arc<SlotAggregatedStatusManager>,
}

impl LlamaCppArbiter {
    pub fn new(
        agent_name: Option<String>,
        applicable_state: AgentApplicableState,
        desired_slots_total: i32,
        slot_aggregated_status_manager: Arc<SlotAggregatedStatusManager>,
    ) -> Self {
        Self {
            agent_name,
            applicable_state,
            desired_slots_total,
            slot_aggregated_status_manager,
        }
    }

    pub async fn spawn(&self) -> Result<LlamaCppArbiterController> {
        let (llamacpp_slot_addr_tx, llamacpp_slot_addr_rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let agent_name_clone = self.agent_name.clone();
        let desired_slots_total = self.desired_slots_total;
        let model_path = self.applicable_state.model_path.clone();
        let slot_aggregated_status_manager = self.slot_aggregated_status_manager.clone();

        let sync_arbiter_thread_handle = thread::spawn(move || -> Result<()> {
            let backend =
                Arc::new(LlamaBackend::init().context("Unable to initialize llama.cpp backend")?);
            let ctx_params = Arc::new(
                LlamaContextParams::default().with_n_ctx(core::num::NonZeroU32::new(4096)),
            );
            let backend_clone = backend.clone();
            let model = Arc::new(
                LlamaModel::load_from_file(
                    &backend_clone.clone(),
                    model_path.clone(),
                    &LlamaModelParams::default(),
                )
                .context("Unable to load model from file")?,
            );

            slot_aggregated_status_manager
                .slot_aggregated_status
                .set_model_path(Some(model_path.display().to_string()));

            let slot_index = Arc::new(AtomicU32::new(0));
            let system = System::new();

            system.block_on(async move {
                llamacpp_slot_addr_tx
                    .send(SyncArbiter::start(
                        desired_slots_total as usize,
                        move || {
                            LlamaCppSlot::new(
                                agent_name_clone.clone(),
                                backend.clone(),
                                ctx_params.clone(),
                                model.clone(),
                                model_path.clone(),
                                slot_index.fetch_add(1, Ordering::SeqCst),
                                slot_aggregated_status_manager.bind_slot_status(),
                            )
                            .expect("Failed to create LlamaCppSlot")
                        },
                    ))
                    .expect("Failed to send LlamaCppSlot address");

                shutdown_rx
                    .await
                    .expect("Failed to receive shutdown signal");

                System::current().stop();
            });

            Ok(())
        });

        Ok(LlamaCppArbiterController::new(
            llamacpp_slot_addr_rx
                .await
                .context("Unable to await for llamacpp slot addr")?,
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
    use crate::agent::generate_tokens_request::GenerateTokensRequest;
    use crate::agent_desired_model::AgentDesiredModel;
    use crate::agent_desired_state::AgentDesiredState;
    use crate::converts_to_applicable_state::ConvertsToApplicableState as _;
    use crate::huggingface_model_reference::HuggingFaceModelReference;
    use crate::request_params::GenerateTokensParams;

    const SLOTS_TOTAL: i32 = 2;

    #[actix_web::test]
    async fn test_llamacpp_arbiter_spawn() -> Result<()> {
        let desired_state = AgentDesiredState {
            model: AgentDesiredModel::HuggingFace(HuggingFaceModelReference {
                filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
                repo: "Qwen/Qwen3-0.6B-GGUF".to_string(),
                // filename: "Qwen3-8B-Q4_K_M.gguf".to_string(),
                // repo: "Qwen/Qwen3-8B-GGUF".to_string(),
            }),
        };
        let slot_aggregated_status_manager =
            Arc::new(SlotAggregatedStatusManager::new(SLOTS_TOTAL));

        let applicable_state = desired_state
            .to_applicable_state()
            .await?
            .expect("Failed to convert to applicable state");

        let llamacpp_arbiter = LlamaCppArbiter::new(
            Some("test_agent".to_string()),
            applicable_state,
            SLOTS_TOTAL,
            slot_aggregated_status_manager,
        );
        let controller = llamacpp_arbiter.spawn().await?;

        let prompt =
            "<|im_start|>user\nHow can I make a cat happy?<|im_end|>\n<|im_start|>assistant\n";
        let (generated_tokens_tx, mut generated_tokens_rx) = mpsc::unbounded_channel();

        let (_, generate_tokens_stop_rx_1) = mpsc::unbounded_channel::<()>();
        let (_, generate_tokens_stop_rx_2) = mpsc::unbounded_channel::<()>();
        let (_, generate_tokens_stop_rx_3) = mpsc::unbounded_channel::<()>();

        let futures = vec![
            controller.llamacpp_slot_addr.send(GenerateTokensRequest {
                generated_tokens_tx: generated_tokens_tx.clone(),
                generate_tokens_stop_rx: generate_tokens_stop_rx_1,
                generate_tokens_params: GenerateTokensParams {
                    max_tokens: 30,
                    prompt: prompt.to_string(),
                },
            }),
            controller.llamacpp_slot_addr.send(GenerateTokensRequest {
                generated_tokens_tx: generated_tokens_tx.clone(),
                generate_tokens_stop_rx: generate_tokens_stop_rx_2,
                generate_tokens_params: GenerateTokensParams {
                    max_tokens: 30,
                    prompt: prompt.to_string(),
                },
            }),
            controller.llamacpp_slot_addr.send(GenerateTokensRequest {
                generated_tokens_tx,
                generate_tokens_stop_rx: generate_tokens_stop_rx_3,
                generate_tokens_params: GenerateTokensParams {
                    max_tokens: 30,
                    prompt: prompt.to_string(),
                },
            }),
        ];

        tokio::spawn(async move {
            while let Some(generated_token) = generated_tokens_rx.recv().await {
                println!("Received generated token: {generated_token:?}");
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
