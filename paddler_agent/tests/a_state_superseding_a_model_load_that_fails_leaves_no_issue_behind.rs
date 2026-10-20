use futures_util::FutureExt as _;
use tokio_util::sync::CancellationToken;

use paddler_agent::agent_applicable_state::AgentApplicableState;
use paddler_blocking_thread_fixture::occupied_blocking_thread::OccupiedBlockingThread;
use paddler_blocking_thread_fixture::single_blocking_thread_runtime::single_blocking_thread_runtime;
use paddler_messaging::agent_issue::AgentIssue;
use paddler_messaging::agent_issue_params::model_path::ModelPath;

use crate::arbiter_under_test::ArbiterUnderTest;
use crate::settings_whose_model_cannot_load::settings_whose_model_cannot_load;
use crate::state_whose_model_cannot_load::state_whose_model_cannot_load;

#[test]
fn a_state_superseding_a_model_load_that_fails_leaves_no_issue_behind() {
    single_blocking_thread_runtime()
        .expect("the test runtime must build")
        .block_on(async {
            let occupied_blocking_thread = OccupiedBlockingThread::occupy();
            let mut arbiter_under_test = ArbiterUnderTest::new(CancellationToken::new());

            arbiter_under_test
                .pipeline_arbiter
                .apply(state_whose_model_cannot_load());
            arbiter_under_test.conclude_next_progress().await;

            assert!(
                arbiter_under_test
                    .pipeline_arbiter
                    .next_finished_startup()
                    .now_or_never()
                    .is_none()
            );

            arbiter_under_test
                .pipeline_arbiter
                .apply(AgentApplicableState::NotConfigured);
            occupied_blocking_thread
                .release()
                .await
                .expect("the blocking thread occupant must be released");
            arbiter_under_test.conclude_next_progress().await;
            arbiter_under_test.conclude_next_progress().await;
            arbiter_under_test.conclude_next_progress().await;

            assert!(
                !arbiter_under_test
                    .inference_runtime_context
                    .slot_aggregated_status
                    .has_issue(&AgentIssue::ModelCannotBeLoaded(ModelPath::from(
                        settings_whose_model_cannot_load().model_path.as_path()
                    )))
            );

            arbiter_under_test.pipeline_arbiter.shut_down().await;
        });
}
