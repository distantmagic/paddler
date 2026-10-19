use std::sync::Arc;

use tokio::sync::watch;

use paddler_messaging::agent_desired_state::AgentDesiredState;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::inference_capability::InferenceCapability;

use crate::balancer_applicable_state::BalancerApplicableState;
use crate::cluster_does_not_serve_inference_capability::ClusterDoesNotServeInferenceCapability;

pub struct BalancerApplicableStateHolder {
    balancer_applicable_state_rx: watch::Receiver<Arc<BalancerApplicableState>>,
}

impl BalancerApplicableStateHolder {
    #[must_use]
    pub const fn new(
        balancer_applicable_state_rx: watch::Receiver<Arc<BalancerApplicableState>>,
    ) -> Self {
        Self {
            balancer_applicable_state_rx,
        }
    }

    #[must_use]
    pub fn get_agent_desired_state(&self) -> AgentDesiredState {
        self.balancer_applicable_state_rx
            .borrow()
            .agent_desired_state
            .clone()
    }

    #[must_use]
    pub fn get_balancer_applicable_state(&self) -> Arc<BalancerApplicableState> {
        self.balancer_applicable_state_rx.borrow().clone()
    }

    #[must_use]
    pub fn get_balancer_desired_state(&self) -> BalancerDesiredState {
        self.balancer_applicable_state_rx
            .borrow()
            .balancer_desired_state
            .clone()
    }

    pub fn require_inference_capability(
        &self,
        requested_inference_capability: InferenceCapability,
    ) -> Result<Arc<BalancerApplicableState>, ClusterDoesNotServeInferenceCapability> {
        let balancer_applicable_state = self.get_balancer_applicable_state();
        let served_inference_mode = balancer_applicable_state.inference_mode();

        if served_inference_mode.serves(requested_inference_capability) {
            Ok(balancer_applicable_state)
        } else {
            Err(ClusterDoesNotServeInferenceCapability {
                requested_inference_capability,
                served_inference_mode,
            })
        }
    }

    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<Arc<BalancerApplicableState>> {
        self.balancer_applicable_state_rx.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::watch;

    use paddler_messaging::balancer_desired_state::BalancerDesiredState;
    use paddler_messaging::inference_capability::InferenceCapability;
    use paddler_messaging::inference_mode::InferenceMode;

    use super::BalancerApplicableStateHolder;
    use crate::balancer_applicable_state::BalancerApplicableState;
    use crate::cluster_does_not_serve_inference_capability::ClusterDoesNotServeInferenceCapability;

    fn holder_serving_text_generation_and_embeddings() -> BalancerApplicableStateHolder {
        let (_balancer_applicable_state_tx, balancer_applicable_state_rx) = watch::channel(
            Arc::new(BalancerApplicableState::from(BalancerDesiredState {
                inference_mode: InferenceMode::TextGenerationAndEmbeddings,
                ..BalancerDesiredState::default()
            })),
        );

        BalancerApplicableStateHolder::new(balancer_applicable_state_rx)
    }

    #[test]
    fn grants_a_capability_the_served_mode_includes() {
        assert_eq!(
            holder_serving_text_generation_and_embeddings()
                .require_inference_capability(InferenceCapability::Embeddings)
                .map(|balancer_applicable_state| balancer_applicable_state.inference_mode()),
            Ok(InferenceMode::TextGenerationAndEmbeddings)
        );
    }

    #[test]
    fn refuses_a_capability_the_served_mode_lacks() {
        assert_eq!(
            holder_serving_text_generation_and_embeddings()
                .require_inference_capability(InferenceCapability::Decisions)
                .err(),
            Some(ClusterDoesNotServeInferenceCapability {
                requested_inference_capability: InferenceCapability::Decisions,
                served_inference_mode: InferenceMode::TextGenerationAndEmbeddings,
            })
        );
    }
}
