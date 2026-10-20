use futures_util::FutureExt as _;
use tokio::join;
use tokio_util::sync::CancellationToken;

use paddler_blocking_thread_fixture::occupied_blocking_thread::OccupiedBlockingThread;
use paddler_blocking_thread_fixture::single_blocking_thread_runtime::single_blocking_thread_runtime;
use paddler_messaging::agent_issue::AgentIssue;
use paddler_messaging::agent_issue_params::slots_insufficient_for_decisions_params::SlotsInsufficientForDecisionsParams;
use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;

use crate::arbiter_under_test::ArbiterUnderTest;
use crate::state_enabling_decisions::state_enabling_decisions;

#[test]
fn shutting_down_waits_for_a_decisions_preparation_that_fails() {
    single_blocking_thread_runtime()
        .expect("the test runtime must build")
        .block_on(async {
            let occupied_blocking_thread = OccupiedBlockingThread::occupy();
            let mut arbiter_under_test = ArbiterUnderTest::new(CancellationToken::new());

            arbiter_under_test
                .pipeline_arbiter
                .apply(state_enabling_decisions());

            assert!(
                arbiter_under_test
                    .pipeline_arbiter
                    .next_finished_startup()
                    .now_or_never()
                    .is_none()
            );

            let ArbiterUnderTest {
                inference_runtime_context,
                pipeline_arbiter,
                ..
            } = arbiter_under_test;

            join!(pipeline_arbiter.shut_down(), async {
                occupied_blocking_thread
                    .release()
                    .await
                    .expect("the blocking thread occupant must be released");
            });

            assert!(inference_runtime_context.slot_aggregated_status.has_issue(
                &AgentIssue::SlotsInsufficientForDecisions(SlotsInsufficientForDecisionsParams {
                    desired_slots: 1,
                    required_slots: DECISION_SLOTS_MINIMUM.get(),
                })
            ));
        });
}
