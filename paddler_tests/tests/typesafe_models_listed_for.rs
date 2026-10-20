use anyhow::Result;

use paddler_messaging::agent_desired_model::AgentDesiredModel;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_test_cluster_harness::cluster_desired_state::ClusterDesiredState;
use paddler_test_cluster_harness::cluster_params::ClusterParams;
use paddler_test_cluster_harness::typesafe_api_response::TypeSafeApiResponse;

use crate::cluster_without_agents_serving::cluster_without_agents_serving;
use crate::start_cluster::start_cluster;

pub async fn typesafe_models_listed_for(model: AgentDesiredModel) -> Result<TypeSafeApiResponse> {
    let cluster = start_cluster(ClusterParams {
        desired_state: ClusterDesiredState::Apply(Box::new(BalancerDesiredState {
            model,
            inference_mode: InferenceMode::TextGenerationEmbeddingsAndDecisions,
            ..BalancerDesiredState::default()
        })),
        ..cluster_without_agents_serving(InferenceMode::TextGenerationEmbeddingsAndDecisions)
    })
    .await?;
    let typesafe_models = cluster.typesafe_models().await?;

    cluster.shutdown().await?;

    Ok(typesafe_models)
}
