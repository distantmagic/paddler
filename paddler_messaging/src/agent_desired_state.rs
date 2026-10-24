use serde::Deserialize;
use serde::Serialize;

use paddler_inference_parameters::embedding_parameters::EmbeddingParameters;
use paddler_inference_parameters::model_runtime_parameters::ModelRuntimeParameters;
use paddler_inference_parameters::pooling_type::PoolingType;

use crate::agent_decisions_settings::AgentDecisionsSettings;
use crate::agent_desired_model::AgentDesiredModel;
use crate::balancer_desired_state::BalancerDesiredState;
use crate::balancer_text_generation_settings::BalancerTextGenerationSettings;
use crate::inference_mode::InferenceMode;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentDesiredState {
    pub decisions: AgentDecisionsSettings,
    pub model: AgentDesiredModel,
    pub model_runtime_parameters: ModelRuntimeParameters,
    pub pooling_type: PoolingType,
    pub text_generation: BalancerTextGenerationSettings,
}

impl Default for AgentDesiredState {
    fn default() -> Self {
        Self::from(BalancerDesiredState::default())
    }
}

impl From<BalancerDesiredState> for AgentDesiredState {
    fn from(
        BalancerDesiredState {
            decision,
            embeddings: EmbeddingParameters { pooling_type, .. },
            inference_mode,
            model,
            model_runtime_parameters,
            text_generation,
        }: BalancerDesiredState,
    ) -> Self {
        Self {
            decisions: match inference_mode {
                InferenceMode::TextGenerationAndEmbeddings => AgentDecisionsSettings::NotServed,
                InferenceMode::TextGenerationEmbeddingsAndDecisions => {
                    AgentDecisionsSettings::Served(decision)
                }
            },
            model,
            model_runtime_parameters,
            pooling_type,
            text_generation,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use paddler_inference_parameters::embedding_parameters::EmbeddingParameters;
    use paddler_inference_parameters::pooling_type::PoolingType;

    use super::AgentDesiredState;
    use crate::agent_decisions_settings::AgentDecisionsSettings;
    use crate::agent_desired_model::AgentDesiredModel;
    use crate::balancer_desired_state::BalancerDesiredState;
    use crate::decision_settings::DecisionSettings;
    use crate::inference_mode::InferenceMode;

    fn decision_settings() -> DecisionSettings {
        DecisionSettings {
            pointer_head: AgentDesiredModel::Uri("pointer_head.gguf".to_owned()),
        }
    }

    fn agent_decisions_while_serving(inference_mode: InferenceMode) -> AgentDecisionsSettings {
        AgentDesiredState::from(BalancerDesiredState {
            decision: decision_settings(),
            inference_mode,
            ..BalancerDesiredState::default()
        })
        .decisions
    }

    #[test]
    fn agents_receive_the_decision_settings_only_while_the_cluster_serves_decisions() {
        assert_eq!(
            agent_decisions_while_serving(InferenceMode::TextGenerationAndEmbeddings),
            AgentDecisionsSettings::NotServed
        );
        assert_eq!(
            agent_decisions_while_serving(InferenceMode::TextGenerationEmbeddingsAndDecisions),
            AgentDecisionsSettings::Served(decision_settings())
        );
    }

    #[test]
    fn the_balancer_only_embedding_batch_size_does_not_reach_agents() {
        let desired_state_batching = |embedding_batch_size| {
            AgentDesiredState::from(BalancerDesiredState {
                embeddings: EmbeddingParameters {
                    embedding_batch_size,
                    pooling_type: PoolingType::Mean,
                },
                ..BalancerDesiredState::default()
            })
        };

        assert_eq!(
            desired_state_batching(NonZeroUsize::MIN),
            desired_state_batching(NonZeroUsize::MAX)
        );
    }
}
