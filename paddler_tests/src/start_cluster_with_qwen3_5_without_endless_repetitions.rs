use anyhow::Result;

use paddler_inference_parameters::sampling_parameters::SamplingParameters;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::balancer_text_generation_settings::BalancerTextGenerationSettings;
use paddler_test_cluster_harness::agent_config::AgentConfig;
use paddler_test_cluster_harness::cluster::Cluster;
use paddler_test_cluster_harness::cluster_desired_state::ClusterDesiredState;
use paddler_test_cluster_harness::cluster_params::ClusterParams;
use paddler_test_cluster_harness::model_card::qwen3_5_0_8b::qwen3_5_0_8b;

use crate::start_cluster::start_cluster;

const LLAMA_CPP_DEFAULT_PENALTY_LAST_N: i32 = 64;
const QWEN_RECOMMENDED_PRESENCE_PENALTY: f32 = 1.5;

pub async fn start_cluster_with_qwen3_5_without_endless_repetitions(
    agents: Vec<AgentConfig>,
) -> Result<Cluster> {
    let desired_state = qwen3_5_0_8b().into_desired_state();

    start_cluster(ClusterParams {
        agents,
        desired_state: ClusterDesiredState::Apply(Box::new(BalancerDesiredState {
            text_generation: BalancerTextGenerationSettings {
                sampling_parameters: SamplingParameters {
                    penalty_last_n: LLAMA_CPP_DEFAULT_PENALTY_LAST_N,
                    penalty_presence: QWEN_RECOMMENDED_PRESENCE_PENALTY,
                    ..SamplingParameters::deterministic()
                },
                ..desired_state.text_generation
            },
            ..desired_state
        })),
        wait_for_slots_ready: true,
        ..ClusterParams::default()
    })
    .await
}
