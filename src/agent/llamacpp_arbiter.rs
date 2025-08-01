use core::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use actix::sync::SyncArbiter;
use actix::System;
use anyhow::anyhow;
use anyhow::Context as _;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::Special;
use log::error;
use tokio::sync::oneshot;

use crate::agent::llamacpp_arbiter_controller::LlamaCppArbiterController;
use crate::agent::llamacpp_slot::LlamaCppSlot;
use crate::agent::model_metadata_holder::ModelMetadataHolder;
use crate::agent_issue::AgentIssue;
use crate::agent_issue_fix::AgentIssueFix;
use crate::inference_parameters::InferenceParameters;
use crate::model_metadata::ModelMetadata;
use crate::slot_aggregated_status_manager::SlotAggregatedStatusManager;
use crate::chat_template::ChatTemplate;

pub struct LlamaCppArbiter {
    agent_name: Option<String>,
    desired_slots_total: i32,
    inference_parameters: InferenceParameters,
    model_metadata_holder: Arc<ModelMetadataHolder>,
    model_path: PathBuf,
    override_chat_template: Option<ChatTemplate>,
    slot_aggregated_status_manager: Arc<SlotAggregatedStatusManager>,
}

impl LlamaCppArbiter {
    pub fn new(
        agent_name: Option<String>,
        desired_slots_total: i32,
        inference_parameters: InferenceParameters,
        model_metadata_holder: Arc<ModelMetadataHolder>,
        model_path: PathBuf,
        override_chat_template: Option<ChatTemplate>,
        slot_aggregated_status_manager: Arc<SlotAggregatedStatusManager>,
    ) -> Self {
        Self {
            agent_name,
            desired_slots_total,
            inference_parameters,
            model_metadata_holder,
            model_path,
            override_chat_template,
            slot_aggregated_status_manager,
        }
    }

    pub async fn spawn(&self) -> Result<LlamaCppArbiterController> {
        let model_path_string = self.model_path.display().to_string();

        if self
            .slot_aggregated_status_manager
            .slot_aggregated_status
            .has_issue(&AgentIssue::UnableToFindChatTemplate(
                model_path_string.clone(),
            ))
        {
            return Err(anyhow!(
                "Unable to establish chat template for model at path: {model_path_string}"
            ));
        }

        let (chat_template_loaded_tx, chat_template_loaded_rx) = oneshot::channel::<()>();
        let (llamacpp_slot_addr_tx, llamacpp_slot_addr_rx) = oneshot::channel();
        let (model_loaded_tx, model_loaded_rx) = oneshot::channel::<()>();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let agent_name_clone = self.agent_name.clone();
        let desired_slots_total = self.desired_slots_total;
        let inference_parameters = self.inference_parameters.clone();
        let model_metadata_holder = self.model_metadata_holder.clone();
        let model_path = self.model_path.clone();
        let model_path_string_clone = model_path_string.clone();
        let override_chat_template = self.override_chat_template.clone();
        let slot_aggregated_status_manager = self.slot_aggregated_status_manager.clone();

        let sync_arbiter_thread_handle = thread::spawn(move || -> Result<()> {
            let backend = Arc::new(LlamaBackend::init().context("Unable to initialize llama.cpp backend")?);
            let ctx_params = Arc::new(LlamaContextParams::default().with_n_ctx(NonZeroU32::new(inference_parameters.context_size)));
            let backend_clone = backend.clone();
            let model = Arc::new(
                LlamaModel::load_from_file(
                    &backend_clone.clone(),
                    model_path.clone(),
                    &LlamaModelParams::default(),
                )
                .context("Unable to load model from file")?,
            );

            if model_loaded_tx.send(()).is_err() {
                let message = format!(
                    "Failed to send model loaded signal for model at path: {}",
                    model_path.display()
                );

                error!("{message}");

                return Err(anyhow!(message));
            }

            let mut model_metadata = ModelMetadata::new();

            for i in 0..model.meta_count() {
                model_metadata
                    .set_meta_field(model.meta_key_by_index(i)?, model.meta_val_str_by_index(i)?);
            }

            model_metadata_holder.set_model_metadata(model_metadata);

            let llama_chat_template_string = match override_chat_template {
                Some(chat_template) => chat_template.content,
                None => model
                    .chat_template(None)
                    .context(format!(
                        "Failed to load chat template for model at path: {}",
                        model_path.display()
                    ))?
                    .to_string()?,
            };

            if chat_template_loaded_tx.send(()).is_err() {
                let message = format!(
                    "Failed to send chat template loaded signal for model at path: {}",
                    model_path.display()
                );

                error!("{message}");

                return Err(anyhow!(message));
            }

            slot_aggregated_status_manager
                .slot_aggregated_status
                .set_model_path(Some(model_path_string_clone));

            let slot_index = Arc::new(AtomicU32::new(0));
            let system = System::new();
            let token_bos_str = model.token_to_str(model.token_bos(), Special::Tokenize)?;
            let token_nl_str = model.token_to_str(model.token_nl(), Special::Tokenize)?;
            let token_eos_str = model.token_to_str(model.token_eos(), Special::Tokenize)?;

            system.block_on(async move {
                llamacpp_slot_addr_tx
                    .send(SyncArbiter::start(
                        desired_slots_total as usize,
                        move || {
                            LlamaCppSlot::new(
                                agent_name_clone.clone(),
                                backend.clone(),
                                ctx_params.clone(),
                                inference_parameters.clone(),
                                llama_chat_template_string.clone(),
                                model.clone(),
                                model_path.clone(),
                                slot_index.fetch_add(1, Ordering::SeqCst),
                                slot_aggregated_status_manager.bind_slot_status(),
                                token_bos_str.clone(),
                                token_eos_str.clone(),
                                token_nl_str.clone(),
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

        match model_loaded_rx
            .await
            .context("Failed to receive model loaded signal")
        {
            Ok(()) => {
                self.slot_aggregated_status_manager
                    .slot_aggregated_status
                    .register_fix(AgentIssueFix::ModelIsLoaded);
            }
            Err(err) => {
                error!("Failed to load model: {err}");

                self.slot_aggregated_status_manager
                    .slot_aggregated_status
                    .register_issue(AgentIssue::ModelCannotBeLoaded(model_path_string.clone()));
            }
        }

        match chat_template_loaded_rx
            .await
            .context("Failed to receive chat template loaded signal")
        {
            Ok(()) => {
                self.slot_aggregated_status_manager
                    .slot_aggregated_status
                    .register_fix(AgentIssueFix::ModelChatTemplateIsLoaded);
            }
            Err(err) => {
                error!("Failed to load chat template: {err}");

                if !self
                    .slot_aggregated_status_manager
                    .slot_aggregated_status
                    .has_issue(&AgentIssue::ModelCannotBeLoaded(model_path_string.clone()))
                {
                    // If the model cannot be loaded, that doesn't mean that the chat template
                    // cannot be loaded.
                    self.slot_aggregated_status_manager
                        .slot_aggregated_status
                        .register_issue(AgentIssue::UnableToFindChatTemplate(
                            model_path_string.clone(),
                        ));
                }
            }
        }

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
    use crate::inference_parameters::InferenceParameters;
    use crate::request_params::GenerateTokensParams;

    const SLOTS_TOTAL: i32 = 2;

    #[actix_web::test]
    async fn test_llamacpp_arbiter_spawn() -> Result<()> {
        let desired_state = AgentDesiredState {
            model: AgentDesiredModel::HuggingFace(HuggingFaceModelReference {
                filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
                repo_id: "Qwen/Qwen3-0.6B-GGUF".to_string(),
                revision: "main".to_string(),
            }),
            inference_parameters: InferenceParameters::default(),
        };
        let slot_aggregated_status_manager =
            Arc::new(SlotAggregatedStatusManager::new(SLOTS_TOTAL));

        let applicable_state = desired_state
            .to_applicable_state()
            .await?
            .expect("Failed to convert to applicable state");

        let llamacpp_arbiter = LlamaCppArbiter::new(
            Some("test_agent".to_string()),
            SLOTS_TOTAL,
            applicable_state.inference_parameters,
            Arc::new(ModelMetadataHolder::new()),
            applicable_state.model_path.expect("Model path is required"),
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
