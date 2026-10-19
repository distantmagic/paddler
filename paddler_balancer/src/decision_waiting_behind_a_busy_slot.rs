use std::sync::Arc;
use std::time::Duration;

use tokio::spawn;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;
use paddler_messaging::inference_capability::InferenceCapability;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_messaging::subscribes_to_updates::SubscribesToUpdates as _;

use crate::agent_controller::AgentController;
use crate::agent_controller_fixture::AgentControllerFixture;
use crate::agent_controller_pool::AgentControllerPool;
use crate::agent_controller_registration::AgentControllerRegistration;
use crate::buffered_request_agent_wait_result::BufferedRequestAgentWaitResult;
use crate::buffered_request_manager::BufferedRequestManager;
use crate::decision_with_leading_questions_requirement::DECISION_WITH_LEADING_QUESTIONS_REQUIREMENT;
use crate::dispatched_agent::DispatchedAgent;
use crate::one_slot_of::one_slot_of;

pub struct DecisionWaitingBehindABusySlot {
    pub agent_controller: Arc<AgentController>,
    pub _agent_controller_registration: AgentControllerRegistration,
    pub buffered_request_manager: Arc<BufferedRequestManager>,
    pub _busy_slot: DispatchedAgent,
    pub decision_cancellation: CancellationToken,
    pub waiting_decision: JoinHandle<Option<BufferedRequestAgentWaitResult>>,
}

impl DecisionWaitingBehindABusySlot {
    pub async fn start(buffer_capacity: u64) -> Self {
        let agent_controller_pool = Arc::new(AgentControllerPool::default());
        let agent_controller = AgentControllerFixture::serving(
            "decisions",
            InferenceMode::TextGenerationEmbeddingsAndDecisions,
            u64::from(DECISION_SLOTS_MINIMUM.get()),
        )
        .agent_controller;
        let agent_controller_registration =
            agent_controller_pool.register_agent_controller(agent_controller.clone());
        let buffered_request_manager = Arc::new(BufferedRequestManager::new(
            agent_controller_pool,
            Duration::MAX,
            buffer_capacity,
        ));
        let busy_slot = buffered_request_manager
            .take_available_agent(one_slot_of(InferenceCapability::TextGeneration))
            .expect("one of the agent's slots must be free");
        let decision_cancellation = CancellationToken::new();
        let waiting_decision = spawn({
            let buffered_request_manager = buffered_request_manager.clone();
            let decision_cancellation = decision_cancellation.clone();

            async move {
                decision_cancellation
                    .run_until_cancelled(
                        buffered_request_manager
                            .wait_for_available_agent(DECISION_WITH_LEADING_QUESTIONS_REQUIREMENT),
                    )
                    .await
            }
        });

        buffered_request_manager
            .subscribe_to_updates()
            .wait_for(|()| buffered_request_manager.buffered_request_counter.get() == 1)
            .await
            .expect("the buffered request manager must keep announcing updates");

        Self {
            agent_controller,
            _agent_controller_registration: agent_controller_registration,
            buffered_request_manager,
            _busy_slot: busy_slot,
            decision_cancellation,
            waiting_decision,
        }
    }

    pub async fn leave_the_waiting_line(&mut self) {
        self.decision_cancellation.cancel();

        assert!(
            (&mut self.waiting_decision)
                .await
                .expect("the waiting decision must not panic")
                .is_none()
        );
    }
}
