use std::path::PathBuf;
use std::sync::mpsc::channel;

use futures_util::FutureExt as _;
use tokio::join;
use tokio::runtime::Builder;
use tokio::task::spawn_blocking;
use tokio_util::sync::CancellationToken;

use paddler_agent::agent_applicable_state::AgentApplicableState;
use paddler_agent::applicable_decisions::ApplicableDecisions;
use paddler_agent::inference_capabilities::InferenceCapabilities;
use paddler_agent::model_serving_settings::ModelServingSettings;
use paddler_messaging::agent_issue::AgentIssue;
use paddler_messaging::agent_issue_params::slots_insufficient_for_decisions_params::SlotsInsufficientForDecisionsParams;
use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;

use crate::arbiter_under_test::ArbiterUnderTest;
use crate::settings_whose_model_cannot_load::settings_whose_model_cannot_load;

const ONLY_BLOCKING_THREAD: usize = 1;

#[test]
fn shutting_down_waits_for_a_decisions_preparation_that_fails() {
    Builder::new_current_thread()
        .max_blocking_threads(ONLY_BLOCKING_THREAD)
        .enable_all()
        .build()
        .expect("the test runtime must build")
        .block_on(async {
            let (release_blocking_thread_tx, release_blocking_thread_rx) = channel::<()>();
            let blocking_thread_occupant =
                spawn_blocking(move || release_blocking_thread_rx.recv());
            let mut arbiter_under_test = ArbiterUnderTest::new(CancellationToken::new());
            let settings_whose_model_cannot_load = settings_whose_model_cannot_load();

            arbiter_under_test
                .pipeline_arbiter
                .apply(AgentApplicableState::WithModel(ModelServingSettings {
                    capabilities: InferenceCapabilities {
                        decisions: ApplicableDecisions::Enabled {
                            pointer_head_path: PathBuf::from("/nonexistent/pointer_head.gguf"),
                        },
                        ..settings_whose_model_cannot_load.capabilities
                    },
                    ..settings_whose_model_cannot_load
                }));

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
                release_blocking_thread_tx
                    .send(())
                    .expect("the blocking thread occupant must be waiting");
            });
            blocking_thread_occupant
                .await
                .expect("the blocking thread occupant must not panic")
                .expect("the blocking thread occupant must be released");

            assert!(inference_runtime_context.slot_aggregated_status.has_issue(
                &AgentIssue::SlotsInsufficientForDecisions(SlotsInsufficientForDecisionsParams {
                    desired_slots: 1,
                    required_slots: DECISION_SLOTS_MINIMUM.get(),
                })
            ));
        });
}
