use std::sync::Arc;

use llama_cpp_bindings::context::LlamaContext;
use llama_cpp_bindings::context::params::LlamaContextParams;
use llama_cpp_bindings::llama_backend::LlamaBackend;
use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::model::LlamaModel;
use llama_cpp_bindings::token::LlamaToken;
use log::warn;
use tokio_util::sync::CancellationToken;

use paddler_agent_status::agent_issue_fix::AgentIssueFix;
use paddler_inference_parameters::model_runtime_parameters::ModelRuntimeParameters;
use paddler_messaging::agent_issue::AgentIssue;
use paddler_messaging::agent_issue_params::slot_cannot_start_params::SlotCannotStartParams;
use paddler_messaging::inference_capability::InferenceCapability;
use paddler_messaging::model_metadata::ModelMetadata;

use crate::agent_runtime_error::AgentRuntimeError;
use crate::describe_error_with_causes::describe_error_with_causes;
use crate::inference_runtime_context::InferenceRuntimeContext;
use crate::inference_thread_count::inference_thread_count;
use crate::llama_context_settings::LlamaContextSettings;
use crate::llama_model_settings::LlamaModelSettings;
use crate::spawn_blocking_resuming_panic::spawn_blocking_resuming_panic;
use crate::startup_outcome::StartupOutcome;

fn read_model_metadata(model: &LlamaModel) -> Result<ModelMetadata, AgentRuntimeError> {
    let mut model_metadata = ModelMetadata::default();

    for metadata_index in 0..model.meta_count() {
        model_metadata.set_meta_field(
            model
                .meta_key_by_index(metadata_index)
                .map_err(AgentRuntimeError::ModelMetadataUnreadable)?,
            model
                .meta_val_str_by_index(metadata_index)
                .map_err(AgentRuntimeError::ModelMetadataUnreadable)?,
        );
    }

    Ok(model_metadata)
}

#[derive(Clone)]
pub struct LoadedLlamaModel {
    pub inference_thread_count: i32,
    pub llama_backend: Arc<LlamaBackend>,
    pub llama_model_settings: LlamaModelSettings,
    pub model: Arc<LlamaModel>,
    pub model_metadata: Arc<ModelMetadata>,
}

impl LoadedLlamaModel {
    pub fn load(
        inference_runtime_context: &InferenceRuntimeContext,
        llama_model_settings: LlamaModelSettings,
    ) -> Result<Self, AgentRuntimeError> {
        let inference_thread_count =
            inference_thread_count(inference_runtime_context.available_parallelism.as_ref())?;
        let model_issue_path = llama_model_settings.reported_model_path();
        let loaded = Self::load_from_file(
            inference_runtime_context,
            inference_thread_count,
            llama_model_settings,
        );

        match &loaded {
            Ok(_loaded_llama_model) => {
                inference_runtime_context
                    .slot_aggregated_status
                    .register_fix(&AgentIssueFix::ModelIsLoaded(model_issue_path));
            }
            Err(_load_failure) => {
                inference_runtime_context
                    .slot_aggregated_status
                    .register_pipeline_issue(AgentIssue::ModelCannotBeLoaded(model_issue_path));
            }
        }

        loaded
    }

    pub async fn load_unless_shutting_down(
        cancellation_token: &CancellationToken,
        inference_runtime_context: InferenceRuntimeContext,
        llama_model_settings: LlamaModelSettings,
    ) -> Result<StartupOutcome<Self>, AgentRuntimeError> {
        if cancellation_token.is_cancelled() {
            return Ok(StartupOutcome::Cancelled);
        }

        let loaded_llama_model = spawn_blocking_resuming_panic(move || {
            Self::load(&inference_runtime_context, llama_model_settings)
        })
        .await?;

        if cancellation_token.is_cancelled() {
            loaded_llama_model.release().await;

            return Ok(StartupOutcome::Cancelled);
        }

        Ok(StartupOutcome::Ready(loaded_llama_model))
    }

    pub async fn release(self) {
        spawn_blocking_resuming_panic(move || drop(self)).await;
    }

    #[must_use]
    pub const fn llama_context_settings<'parameters>(
        &self,
        model_runtime_parameters: &'parameters ModelRuntimeParameters,
        n_seq_max: u32,
    ) -> LlamaContextSettings<'parameters> {
        LlamaContextSettings {
            model_runtime_parameters,
            n_seq_max,
            thread_count: self.inference_thread_count,
        }
    }

    pub fn create_llama_context(
        &self,
        inference_runtime_context: &InferenceRuntimeContext,
        inference_capability: InferenceCapability,
        llama_context_params: LlamaContextParams,
    ) -> Result<LlamaContext<'_>, AgentRuntimeError> {
        let n_seq_max = llama_context_params.n_seq_max();

        LlamaContext::from_model(&self.model, &self.llama_backend, llama_context_params).map_err(
            |source| {
                let context_creation_failure = AgentRuntimeError::ContextCreationFailed(source);
                let error = describe_error_with_causes(&context_creation_failure);

                for slot_index in 0..n_seq_max {
                    inference_runtime_context
                        .slot_aggregated_status
                        .register_pipeline_issue(AgentIssue::SlotCannotStart(
                            SlotCannotStartParams {
                                error: error.clone(),
                                inference_capability,
                                slot_index,
                            },
                        ));
                }

                context_creation_failure
            },
        )
    }

    pub fn warm_up_llama_context(
        &self,
        llama_context: &mut LlamaContext<'_>,
        warmup_batch: &mut LlamaBatch<'static>,
        desired_slots_total: u16,
    ) {
        let warmup_tokens: Vec<LlamaToken> = [self.model.token_bos(), self.model.token_eos()]
            .into_iter()
            .flatten()
            .collect();

        for sequence_index in 0..i32::from(desired_slots_total) {
            if let Err(batch_add_error) =
                warmup_batch.add_sequence(&warmup_tokens, sequence_index, true)
            {
                warn!("Warmup batch add_sequence failed: {batch_add_error:#}");
                return;
            }
        }

        llama_context.clear_kv_cache();
        if let Err(decode_error) = llama_context.decode(warmup_batch) {
            warn!("Warmup decode failed: {decode_error:#}");
        }
        llama_context.synchronize();
        llama_context.clear_kv_cache();
    }

    fn load_from_file(
        inference_runtime_context: &InferenceRuntimeContext,
        inference_thread_count: i32,
        llama_model_settings: LlamaModelSettings,
    ) -> Result<Self, AgentRuntimeError> {
        let llama_backend = inference_runtime_context
            .llama_backend
            .get_or_init()
            .map_err(AgentRuntimeError::BackendInitializationFailed)?;
        let model = LlamaModel::load_from_file(
            &llama_backend,
            &llama_model_settings.model_path,
            &llama_model_settings.to_llama_model_params(),
        )
        .map_err(|source| AgentRuntimeError::ModelLoadFailed {
            model_path: llama_model_settings.model_path.clone(),
            source,
        })?;
        let model_metadata = read_model_metadata(&model)?;

        Ok(Self {
            inference_thread_count,
            llama_backend,
            llama_model_settings,
            model: Arc::new(model),
            model_metadata: Arc::new(model_metadata),
        })
    }
}
