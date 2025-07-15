use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::agent::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::agent::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::agent::llamacpp_arbiter::LlamaCppArbiter;
use crate::agent::llamacpp_arbiter_controller::LlamaCppArbiterController;
use crate::agent::message::GenerateTokens;
use crate::agent::slot_aggregated_metrics_manager::SlotAggregatedMetricsManager;
use crate::service::Service;

pub struct LlamaCppArbiterService {
    llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
    llamacpp_arbiter_controller: Option<LlamaCppArbiterController>,
    generate_tokens_rx: mpsc::Receiver<GenerateTokens>,
    slot_aggregated_metrics_manager: Arc<SlotAggregatedMetricsManager>,
    slots_total: usize,
}

impl LlamaCppArbiterService {
    pub async fn new(
        generate_tokens_rx: mpsc::Receiver<GenerateTokens>,
        llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
        slot_aggregated_metrics_manager: Arc<SlotAggregatedMetricsManager>,
        slots_total: usize,
    ) -> Result<Self> {
        Ok(LlamaCppArbiterService {
            llamacpp_applicable_state_holder,
            llamacpp_arbiter_controller: None,
            generate_tokens_rx,
            slot_aggregated_metrics_manager,
            slots_total,
        })
    }

    async fn on_reconciled_state_change(
        &mut self,
        llamacpp_applicable_state: Option<LlamaCppApplicableState>,
    ) -> Result<()> {
        if let Some(llamacpp_arbiter_controller) = self.llamacpp_arbiter_controller.take() {
            llamacpp_arbiter_controller.shutdown().await?;
        }

        if let Some(llamacpp_applicable_state) = llamacpp_applicable_state {
            self.slot_aggregated_metrics_manager.reset();
            self.llamacpp_arbiter_controller = Some(
                LlamaCppArbiter::new(
                    llamacpp_applicable_state,
                    self.slot_aggregated_metrics_manager.clone(),
                    self.slots_total,
                )
                .spawn()
                .await?,
            );
        }

        Ok(())
    }
}

#[async_trait]
impl Service for LlamaCppArbiterService {
    fn name(&self) -> &'static str {
        "agent::llamacpp_arbiter_service"
    }

    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut reconciled_state = self.llamacpp_applicable_state_holder.subscribe();

        loop {
            tokio::select! {
                _ = shutdown.recv() => return Ok(()),
                generate_tokens = self.generate_tokens_rx.recv() => {
                    match generate_tokens {
                        None => return Ok(()),
                        Some(generate_tokens) => {
                            if let Some(llamacpp_arbiter_controller) = &self.llamacpp_arbiter_controller {
                                llamacpp_arbiter_controller.llamacpp_slot_addr.send(generate_tokens).await??;
                            }
                        }
                    }
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

#[cfg(test)]
#[cfg(feature = "tests_that_use_llms")]
mod tests {
    use anyhow::anyhow;
    use anyhow::Context as _;
    use tokio::sync::oneshot;

    use super::*;
    use crate::agent::converts_to_applicable_state::ConvertsToApplicableState as _;
    use crate::agent::huggingface_model_reference::HuggingFaceModelReference;
    use crate::agent::llamacpp_desired_model::LlamaCppDesiredModel;
    use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;
    use crate::agent::message::GenerateTokens;
    use crate::service_manager::ServiceManager;

    const SLOTS_TOTAL: usize = 2;

    struct MockStateReplacerService {
        applicable_state_ready_tx: Option<oneshot::Sender<()>>,
        llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
    }

    #[async_trait]
    impl Service for MockStateReplacerService {
        fn name(&self) -> &'static str {
            "mock_state_replacer_service"
        }

        async fn run(&mut self, mut _shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
            let desired_state = LlamaCppDesiredState {
                model: LlamaCppDesiredModel::HuggingFace(HuggingFaceModelReference {
                    filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
                    repo: "Qwen/Qwen3-0.6B-GGUF".to_string(),
                }),
            };

            let applicable_state = desired_state.to_applicable_state().await?;

            self.llamacpp_applicable_state_holder
                .set_applicable_state(applicable_state)
                .context("Set new applicable state")?;
            self.applicable_state_ready_tx
                .take()
                .expect("Workaround for one-shot channel ownership")
                .send(())
                .map_err(|_| anyhow!("Failed to send applicable state ready signal"))?;

            Ok(())
        }
    }

    struct MockGenerateTokensRequestService {
        applicable_state_ready_rx: Option<oneshot::Receiver<()>>,
        generate_chunks_ready_tx: Option<oneshot::Sender<()>>,
        generate_tokens_tx: mpsc::Sender<GenerateTokens>,
    }

    #[async_trait]
    impl Service for MockGenerateTokensRequestService {
        fn name(&self) -> &'static str {
            "mock_generate_tokens_request_service"
        }

        async fn run(&mut self, mut _shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
            self.applicable_state_ready_rx
                .take()
                .expect("Workaround for one-shot channel ownership")
                .await?;

            let (chunk_sender, mut chunk_receiver) = mpsc::channel::<String>(100);

            let generate_tokens = GenerateTokens {
                chunk_sender,
                max_tokens: 100,
                prompt: "<|im_start|>user\nHow can I make a cat happy?<|im_end|>\n<|im_start|>assistant\n".to_string(),
            };

            self.generate_tokens_tx
                .send(generate_tokens)
                .await
                .map_err(|_| anyhow!("Failed to send generate tokens request"))?;

            tokio::spawn(async move {
                println!("Awaiting for chunks...");

                while let Some(chunk) = chunk_receiver.recv().await {
                    println!("Received chunk: {chunk}");
                }

                println!("No more chunks to receive, exiting...");
            })
            .await?;

            self.generate_chunks_ready_tx
                .take()
                .expect("Workaround for one-shot channel ownership")
                .send(())
                .map_err(|_| anyhow!("Failed to send generate chunks ready signal"))?;

            Ok(())
        }
    }

    struct MockShutdownService {
        generate_chunks_ready_rx: Option<oneshot::Receiver<()>>,
        shutdown_tx: Option<oneshot::Sender<()>>,
    }

    #[async_trait]
    impl Service for MockShutdownService {
        fn name(&self) -> &'static str {
            "mock_shutdown_service"
        }

        async fn run(&mut self, mut _shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
            self.generate_chunks_ready_rx
                .take()
                .expect("Workaround for one-shot channel ownership")
                .await
                .context("Failed to receive generate chunks ready signal")?;

            println!("MockShutdownService is ready to shutdown");

            self.shutdown_tx
                .take()
                .expect("Workaround for one-shot channel ownership")
                .send(())
                .map_err(|_| anyhow!("Failed to send shutdown signal"))?;

            println!("MockShutdownService sent shutdown signal");

            Ok(())
        }
    }

    #[actix_web::test]
    async fn test_llamacpp_arbiter_service_run() -> Result<()> {
        let (generate_tokens_tx, generate_tokens_rx) = mpsc::channel::<GenerateTokens>(100);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let (applicable_state_ready_tx, applicable_state_ready_rx) = oneshot::channel::<()>();
        let (generate_chunks_ready_tx, generate_chunks_ready_rx) = oneshot::channel::<()>();
        let llamacpp_applicable_state_holder = Arc::new(LlamaCppApplicableStateHolder::new());
        let slot_aggregated_metrics_manager =
            Arc::new(SlotAggregatedMetricsManager::new(SLOTS_TOTAL));

        let mut service_manager = ServiceManager::new();

        service_manager.add_service(
            LlamaCppArbiterService::new(
                generate_tokens_rx,
                llamacpp_applicable_state_holder.clone(),
                slot_aggregated_metrics_manager,
                SLOTS_TOTAL,
            )
            .await?,
        );

        service_manager.add_service(MockStateReplacerService {
            applicable_state_ready_tx: Some(applicable_state_ready_tx),
            llamacpp_applicable_state_holder,
        });

        service_manager.add_service(MockGenerateTokensRequestService {
            applicable_state_ready_rx: Some(applicable_state_ready_rx),
            generate_chunks_ready_tx: Some(generate_chunks_ready_tx),
            generate_tokens_tx,
        });

        service_manager.add_service(MockShutdownService {
            generate_chunks_ready_rx: Some(generate_chunks_ready_rx),
            shutdown_tx: Some(shutdown_tx),
        });

        service_manager.run_forever(shutdown_rx).await
    }
}
