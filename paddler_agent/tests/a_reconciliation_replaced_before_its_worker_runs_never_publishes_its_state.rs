use paddler_agent::agent_applicable_state::AgentApplicableState;
use paddler_messaging::agent_desired_state::AgentDesiredState;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;

use crate::reconciliation_replaced_before_its_worker_runs::ReconciliationAfterReplacement;
use crate::reconciliation_replaced_before_its_worker_runs::ReconciliationReplacedBeforeItsWorkerRuns;

#[test]
fn a_reconciliation_replaced_before_its_worker_runs_never_publishes_its_state() {
    let ReconciliationAfterReplacement {
        agent_applicable_state_holder,
        ..
    } = ReconciliationReplacedBeforeItsWorkerRuns {
        replaced_desired_state: AgentDesiredState::from(BalancerDesiredState::default()),
    }
    .run();

    assert_eq!(
        agent_applicable_state_holder.get_agent_applicable_state(),
        AgentApplicableState::NotConfigured
    );
}
