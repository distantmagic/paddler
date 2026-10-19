#![cfg(feature = "tests_that_use_llms")]

use std::io;
use std::mem::discriminant;
use std::num::NonZeroUsize;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use paddler_agent_runtime::inference_runtime_context::InferenceRuntimeContext;
use paddler_agent_runtime::llama_model_settings::LlamaModelSettings;
use paddler_agent_runtime::loaded_llama_model::LoadedLlamaModel;
use paddler_agent_runtime::reports_available_parallelism::ReportsAvailableParallelism;
use paddler_agent_runtime::startup_outcome::StartupOutcome;
use paddler_agent_runtime::system_parallelism::SystemParallelism;
use paddler_inference_parameters::all_gpu_layers::ALL_GPU_LAYERS;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;

struct ParallelismQueriedAsTheAgentShutsDown {
    agent_shutdown: CancellationToken,
}

impl ReportsAvailableParallelism for ParallelismQueriedAsTheAgentShutsDown {
    fn available_parallelism(&self) -> io::Result<NonZeroUsize> {
        self.agent_shutdown.cancel();

        SystemParallelism.available_parallelism()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_shutdown_while_a_model_loads_cancels_the_load() {
    let agent_shutdown = CancellationToken::new();

    let load_outcome = LoadedLlamaModel::load_unless_shutting_down(
        &agent_shutdown,
        InferenceRuntimeContext {
            available_parallelism: Arc::new(ParallelismQueriedAsTheAgentShutsDown {
                agent_shutdown: agent_shutdown.clone(),
            }),
            ..agent_inference_runtime_context(1)
        },
        LlamaModelSettings {
            model_path: kev_0_8b_target_path("model.gguf"),
            n_gpu_layers: ALL_GPU_LAYERS,
        },
    )
    .await
    .expect("the model must load before the shutdown is noticed");

    assert_eq!(
        discriminant(&load_outcome),
        discriminant(&StartupOutcome::Cancelled)
    );
}
