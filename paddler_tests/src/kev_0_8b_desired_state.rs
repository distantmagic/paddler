use paddler_inference_parameters::all_gpu_layers::ALL_GPU_LAYERS;
use paddler_inference_parameters::model_runtime_parameters::ModelRuntimeParameters;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::decision_settings::DecisionSettings;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_model_source::model_source_error::ModelSourceError;
use paddler_test_cluster_harness::agent_desired_model_at::agent_desired_model_at;

use crate::kev_0_8b_target_path::kev_0_8b_target_path;

pub fn kev_0_8b_desired_state() -> Result<BalancerDesiredState, ModelSourceError> {
    agent_desired_model_at(&kev_0_8b_target_path("pointer_head.gguf")).and_then(|pointer_head| {
        agent_desired_model_at(&kev_0_8b_target_path("model.gguf")).map(|model| {
            BalancerDesiredState {
                decision: DecisionSettings { pointer_head },
                inference_mode: InferenceMode::TextGenerationEmbeddingsAndDecisions,
                model,
                model_runtime_parameters: ModelRuntimeParameters {
                    n_gpu_layers: ALL_GPU_LAYERS,
                    ..ModelRuntimeParameters::default()
                },
                ..BalancerDesiredState::default()
            }
        })
    })
}
