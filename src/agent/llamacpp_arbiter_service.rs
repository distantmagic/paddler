use std::sync::Arc;

use anyhow::Context as _;
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::agent::llamacpp_arbiter::LlamaCppArbiter;
use crate::agent::llamacpp_arbiter_controller::LlamaCppArbiterController;
use crate::agent::message::GenerateTokensChannel;
use crate::agent::slot_aggregated_metrics_manager::SlotAggregatedMetricsManager;
use crate::agent_applicable_state::AgentApplicableState;
use crate::agent_applicable_state_holder::AgentApplicableStateHolder;
use crate::response::ChunkResponse;
use crate::service::Service;

pub struct LlamaCppArbiterService {
    agent_applicable_state_holder: Arc<AgentApplicableStateHolder>,
    generate_tokens_channel_rx: mpsc::Receiver<GenerateTokensChannel>,
    llamacpp_arbiter_controller: Option<LlamaCppArbiterController>,
    slot_aggregated_metrics_manager: Arc<SlotAggregatedMetricsManager>,
    slots_total: i32,
}

impl LlamaCppArbiterService {
    pub async fn new(
        agent_applicable_state_holder: Arc<AgentApplicableStateHolder>,
        generate_tokens_channel_rx: mpsc::Receiver<GenerateTokensChannel>,
        slot_aggregated_metrics_manager: Arc<SlotAggregatedMetricsManager>,
        slots_total: i32,
    ) -> Result<Self> {
        Ok(LlamaCppArbiterService {
            agent_applicable_state_holder,
            generate_tokens_channel_rx,
            llamacpp_arbiter_controller: None,
            slot_aggregated_metrics_manager,
            slots_total,
        })
    }

    async fn on_reconciled_state_change(
        &mut self,
        agent_applicable_state: Option<AgentApplicableState>,
    ) -> Result<()> {
        if let Some(llamacpp_arbiter_controller) = self.llamacpp_arbiter_controller.take() {
            llamacpp_arbiter_controller
                .shutdown()
                .await
                .context("Unable to stop arbiter controller")?;
        }

        if let Some(agent_applicable_state) = agent_applicable_state {
            self.slot_aggregated_metrics_manager.reset();
            self.llamacpp_arbiter_controller = Some(
                LlamaCppArbiter::new(
                    agent_applicable_state,
                    self.slot_aggregated_metrics_manager.clone(),
                    self.slots_total,
                )
                .spawn()
                .await
                .context("Unable to spawn arbiter controller")?,
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
        let mut reconciled_state = self.agent_applicable_state_holder.subscribe();

        loop {
            tokio::select! {
                _ = shutdown.recv() => return Ok(()),
                generate_tokens_channel = self.generate_tokens_channel_rx.recv() => {
                    match generate_tokens_channel {
                        None => return Ok(()),
                        Some(generate_tokens_channel) => {
                            if let Some(llamacpp_arbiter_controller) = &self.llamacpp_arbiter_controller {
                                llamacpp_arbiter_controller.llamacpp_slot_addr.send(generate_tokens_channel).await??;
                            } else {
                                let msg = format!(
                                    "No arbiter available to handle generate tokens request: {:?}",
                                    generate_tokens_channel.params
                                );

                                generate_tokens_channel
                                    .chunk_sender
                                    .send(ChunkResponse::Error(msg.clone()))
                                    .await?;

                                error!("{msg}");
                            }
                        }
                    }
                },
                _ = reconciled_state.changed() => {
                    let agent_applicable_state: Option<AgentApplicableState> = reconciled_state.borrow_and_update().clone();

                    if let Err(err) = self.on_reconciled_state_change(agent_applicable_state).await {
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
    use crate::agent::message::GenerateTokensChannel;
    use crate::agent_desired_model::AgentDesiredModel;
    use crate::agent_desired_state::AgentDesiredState;
    use crate::converts_to_applicable_state::ConvertsToApplicableState as _;
    use crate::huggingface_model_reference::HuggingFaceModelReference;
    use crate::request_params::GenerateTokensParams;
    use crate::service_manager::ServiceManager;

    const SLOTS_TOTAL: i32 = 2;

    struct MockStateReplacerService {
        applicable_state_ready_tx: Option<oneshot::Sender<()>>,
        agent_applicable_state_holder: Arc<AgentApplicableStateHolder>,
    }

    #[async_trait]
    impl Service for MockStateReplacerService {
        fn name(&self) -> &'static str {
            "mock_state_replacer_service"
        }

        async fn run(&mut self, mut _shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
            let desired_state = AgentDesiredState {
                model: AgentDesiredModel::HuggingFace(HuggingFaceModelReference {
                    filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
                    repo: "Qwen/Qwen3-0.6B-GGUF".to_string(),
                }),
            };

            let applicable_state = desired_state.to_applicable_state().await?;

            self.agent_applicable_state_holder
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
        generate_tokens_channel_tx: mpsc::Sender<GenerateTokensChannel>,
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

            let (chunk_sender, mut chunk_receiver) = mpsc::channel::<ChunkResponse>(100);

            let generate_tokens = GenerateTokensChannel {
                chunk_sender,
                params: GenerateTokensParams {
                    max_tokens: 100,
                    prompt: "<|im_start|>user\nHow can I make a cat happy?<|im_end|>\n<|im_start|>assistant\n".to_string(),
                },
            };

            self.generate_tokens_channel_tx
                .send(generate_tokens)
                .await
                .map_err(|_| anyhow!("Failed to send generate tokens request"))?;

            tokio::spawn(async move {
                println!("Awaiting for chunks...");

                while let Some(chunk) = chunk_receiver.recv().await {
                    println!("Received chunk: {chunk:?}");
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
        let (generate_tokens_channel_tx, generate_tokens_channel_rx) =
            mpsc::channel::<GenerateTokensChannel>(100);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let (applicable_state_ready_tx, applicable_state_ready_rx) = oneshot::channel::<()>();
        let (generate_chunks_ready_tx, generate_chunks_ready_rx) = oneshot::channel::<()>();
        let agent_applicable_state_holder = Arc::new(AgentApplicableStateHolder::new());
        let slot_aggregated_metrics_manager =
            Arc::new(SlotAggregatedMetricsManager::new(SLOTS_TOTAL));

        let mut service_manager = ServiceManager::new();

        service_manager.add_service(
            LlamaCppArbiterService::new(
                agent_applicable_state_holder.clone(),
                generate_tokens_channel_rx,
                slot_aggregated_metrics_manager,
                SLOTS_TOTAL,
            )
            .await?,
        );

        service_manager.add_service(MockStateReplacerService {
            applicable_state_ready_tx: Some(applicable_state_ready_tx),
            agent_applicable_state_holder,
        });

        service_manager.add_service(MockGenerateTokensRequestService {
            applicable_state_ready_rx: Some(applicable_state_ready_rx),
            generate_chunks_ready_tx: Some(generate_chunks_ready_tx),
            generate_tokens_channel_tx,
        });

        service_manager.add_service(MockShutdownService {
            generate_chunks_ready_rx: Some(generate_chunks_ready_rx),
            shutdown_tx: Some(shutdown_tx),
        });

        service_manager.run_forever(shutdown_rx).await
    }
}
