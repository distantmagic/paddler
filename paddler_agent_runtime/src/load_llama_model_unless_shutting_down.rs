use tokio_util::sync::CancellationToken;

use crate::agent_runtime_error::AgentRuntimeError;
use crate::inference_runtime_context::InferenceRuntimeContext;
use crate::llama_model_settings::LlamaModelSettings;
use crate::loaded_llama_model::LoadedLlamaModel;
use crate::spawn_blocking_resuming_panic::spawn_blocking_resuming_panic;
use crate::startup_outcome::StartupOutcome;

pub async fn load_llama_model_unless_shutting_down(
    cancellation_token: &CancellationToken,
    inference_runtime_context: InferenceRuntimeContext,
    llama_model_settings: LlamaModelSettings,
) -> Result<StartupOutcome<LoadedLlamaModel>, AgentRuntimeError> {
    if cancellation_token.is_cancelled() {
        return Ok(StartupOutcome::Cancelled);
    }

    let loaded_llama_model = spawn_blocking_resuming_panic(move || {
        LoadedLlamaModel::load(&inference_runtime_context, llama_model_settings)
    })
    .await?;

    if cancellation_token.is_cancelled() {
        loaded_llama_model.release().await;

        return Ok(StartupOutcome::Cancelled);
    }

    Ok(StartupOutcome::Ready(loaded_llama_model))
}
