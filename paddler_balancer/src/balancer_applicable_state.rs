use nanoid::nanoid;

use paddler_messaging::agent_desired_state::AgentDesiredState;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::cluster_inference_mode_report::ClusterInferenceModeReport;
use paddler_messaging::inference_mode::InferenceMode;

#[derive(Clone, Debug)]
pub struct BalancerApplicableState {
    pub agent_desired_state: AgentDesiredState,
    pub balancer_desired_state: BalancerDesiredState,
    pub balancer_instance: String,
    pub version: u64,
}

impl BalancerApplicableState {
    #[must_use]
    pub fn initial(balancer_desired_state: BalancerDesiredState) -> Self {
        Self::of_balancer_instance(balancer_desired_state, nanoid!(), 0)
    }

    #[must_use]
    pub fn superseded_by(&self, balancer_desired_state: BalancerDesiredState) -> Self {
        Self::of_balancer_instance(
            balancer_desired_state,
            self.balancer_instance.clone(),
            self.version + 1,
        )
    }

    #[must_use]
    pub fn cluster_inference_mode_report(&self) -> ClusterInferenceModeReport {
        ClusterInferenceModeReport {
            balancer_instance: self.balancer_instance.clone(),
            inference_mode: self.inference_mode(),
            version: self.version,
        }
    }

    #[must_use]
    pub const fn inference_mode(&self) -> InferenceMode {
        self.balancer_desired_state.inference_mode
    }

    fn of_balancer_instance(
        balancer_desired_state: BalancerDesiredState,
        balancer_instance: String,
        version: u64,
    ) -> Self {
        Self {
            agent_desired_state: AgentDesiredState::from(balancer_desired_state.clone()),
            balancer_desired_state,
            balancer_instance,
            version,
        }
    }
}
