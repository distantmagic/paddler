use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use tokio::sync::futures::Notified;
use tokio::sync::watch;

use paddler_messaging::agent_controller_pool_snapshot::AgentControllerPoolSnapshot;
use paddler_messaging::agent_desired_state::AgentDesiredState;
use paddler_messaging::produces_snapshot::ProducesSnapshot;
use paddler_messaging::subscribes_to_updates::SubscribesToUpdates;

use super::agent_controller::AgentController;
use super::agent_controller_pool_total_slots::AgentControllerPoolTotalSlots;
use crate::agent_controller_pool_updates::AgentControllerPoolUpdates;
use crate::agent_controller_registration::AgentControllerRegistration;
use crate::agent_controller_slot_guard::AgentControllerSlotGuard;
use crate::dispatch_candidate::DispatchCandidate;
use crate::dispatch_position::DispatchPosition;
use crate::dispatch_requirement::DispatchRequirement;
use crate::dispatch_waiting_line::DispatchWaitingLine;
use crate::dispatched_agent::DispatchedAgent;
use crate::registered_agent_controller_guard::RegisteredAgentControllerGuard;
use crate::waiting_line_place::WaitingLinePlace;

pub struct AgentControllerPool {
    agents: DashMap<String, Arc<AgentController>>,
    dispatch_waiting_line: DispatchWaitingLine,
    updates: Arc<AgentControllerPoolUpdates>,
}

impl AgentControllerPool {
    #[must_use]
    pub fn select_least_busy_with_capacity(
        &self,
        dispatch_requirement: DispatchRequirement,
        dispatch_position: DispatchPosition,
    ) -> Option<DispatchCandidate> {
        let mut best: Option<DispatchCandidate> = None;

        for entry in &self.agents {
            let agent_controller = entry.value().clone();
            let snapshot = agent_controller.slots_processing.get();
            let agent_runtime_status = agent_controller.reported_status.read().status.runtime;
            let Some(reserved_slots) = dispatch_requirement.reservable_slots(
                &agent_runtime_status,
                snapshot
                    + self
                        .dispatch_waiting_line
                        .slots_held_ahead_of(dispatch_position, &agent_runtime_status),
            ) else {
                continue;
            };

            best = Some(match best {
                Some(current) if current.snapshot <= snapshot => current,
                _ => DispatchCandidate {
                    agent_controller,
                    reserved_slots,
                    snapshot,
                },
            });
        }

        best
    }

    pub fn try_claim(
        &self,
        candidate: DispatchCandidate,
    ) -> Result<DispatchedAgent, DispatchCandidate> {
        if candidate
            .agent_controller
            .slots_processing
            .compare_and_swap(
                candidate.snapshot,
                candidate.snapshot + u64::from(candidate.reserved_slots.get()),
            )
        {
            self.updates.signal();

            let slot_guard = AgentControllerSlotGuard::new(
                candidate.agent_controller.clone(),
                self.updates.clone(),
                candidate.reserved_slots,
            );

            Ok(DispatchedAgent::new(candidate.agent_controller, slot_guard))
        } else {
            Err(candidate)
        }
    }

    pub fn agent_availability_changed(&self) -> Notified<'_> {
        self.updates.agent_availability_changed()
    }

    #[must_use]
    pub fn join_waiting_line(
        &self,
        dispatch_requirement: DispatchRequirement,
    ) -> WaitingLinePlace<'_> {
        WaitingLinePlace {
            dispatch_waiting_line: &self.dispatch_waiting_line,
            ticket: self.dispatch_waiting_line.join(dispatch_requirement),
        }
    }

    pub async fn next_available_agent<'pool>(
        &'pool self,
        mut agent_availability_changed: Notified<'pool>,
        dispatch_requirement: DispatchRequirement,
        dispatch_position: DispatchPosition,
    ) -> DispatchedAgent {
        loop {
            agent_availability_changed.await;
            agent_availability_changed = self.agent_availability_changed();

            if let Some(dispatched_agent) =
                self.take_least_busy_agent_controller(dispatch_requirement, dispatch_position)
            {
                return dispatched_agent;
            }
        }
    }

    #[must_use]
    pub fn take_least_busy_agent_controller(
        &self,
        dispatch_requirement: DispatchRequirement,
        dispatch_position: DispatchPosition,
    ) -> Option<DispatchedAgent> {
        loop {
            let candidate =
                self.select_least_busy_with_capacity(dispatch_requirement, dispatch_position)?;

            if let Ok(dispatched) = self.try_claim(candidate) {
                return Some(dispatched);
            }
        }
    }

    #[must_use]
    pub fn get_agent_controller(&self, agent_id: &str) -> Option<Arc<AgentController>> {
        self.agents.get(agent_id).map(|entry| entry.value().clone())
    }

    pub fn register_agent_controller(
        self: &Arc<Self>,
        agent_controller: Arc<AgentController>,
    ) -> AgentControllerRegistration {
        match self.agents.entry(agent_controller.id.clone()) {
            Entry::Occupied(_registered_agent) => AgentControllerRegistration::DuplicateAgentId,
            Entry::Vacant(vacant_agent_slot) => {
                vacant_agent_slot.insert(agent_controller.clone());
                self.updates.signal();

                AgentControllerRegistration::Registered(RegisteredAgentControllerGuard {
                    agent_controller,
                    agent_controller_pool: self.clone(),
                })
            }
        }
    }

    pub fn remove_agent_controller(&self, agent_id: &str) {
        self.agents.remove(agent_id);
        self.updates.signal();
    }

    pub fn set_desired_state(&self, desired_state: &AgentDesiredState) {
        for agent in &self.agents {
            agent.value().set_desired_state(desired_state.clone());
        }
    }

    #[must_use]
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn signal_update(&self) {
        self.updates.signal();
    }

    #[must_use]
    pub fn total_slots(&self) -> AgentControllerPoolTotalSlots {
        let mut slots_processing = 0;
        let mut slots_total = 0;

        for entry in &self.agents {
            let agent = entry.value();

            slots_processing += agent.slots_processing.get();
            slots_total += agent.reported_status.read().status.runtime.slots_total();
        }

        AgentControllerPoolTotalSlots {
            slots_processing,
            slots_total,
        }
    }
}

impl Default for AgentControllerPool {
    fn default() -> Self {
        let updates = Arc::new(AgentControllerPoolUpdates::default());

        Self {
            agents: DashMap::new(),
            dispatch_waiting_line: DispatchWaitingLine::new(updates.clone()),
            updates,
        }
    }
}

impl SubscribesToUpdates for AgentControllerPool {
    fn subscribe_to_updates(&self) -> watch::Receiver<()> {
        self.updates.subscribe()
    }
}

impl ProducesSnapshot for AgentControllerPool {
    type Snapshot = AgentControllerPoolSnapshot;

    fn make_snapshot(&self) -> Self::Snapshot {
        AgentControllerPoolSnapshot {
            agents: self
                .agents
                .iter()
                .map(|entry| entry.value().make_snapshot())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;
    use std::sync::Arc;

    use paddler_messaging::agent_desired_state::AgentDesiredState;
    use paddler_messaging::balancer_desired_state::BalancerDesiredState;
    use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;
    use paddler_messaging::inference_capability::InferenceCapability;
    use paddler_messaging::inference_mode::InferenceMode;
    use paddler_messaging::management_socket::agent::message::Message as AgentJsonRpcMessage;
    use paddler_messaging::management_socket::agent::notification::Notification as AgentJsonRpcNotification;
    use paddler_messaging::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;

    use super::AgentControllerPool;
    use crate::agent_controller_fixture::AgentControllerFixture;
    use crate::agent_controller_registration::AgentControllerRegistration;
    use crate::dispatch_position::DispatchPosition;
    use crate::dispatch_requirement::DispatchRequirement;
    use crate::slot_demand::SlotDemand;

    fn one_slot_of(inference_capability: InferenceCapability) -> DispatchRequirement {
        DispatchRequirement {
            inference_capability,
            slot_demand: SlotDemand::Exactly(NonZeroU16::MIN),
        }
    }

    fn agent_with_inbox(agent_id: &str) -> AgentControllerFixture {
        AgentControllerFixture::reporting(agent_id, SlotAggregatedStatusSnapshot::default())
    }

    #[test]
    fn dispatches_only_to_agents_with_free_slots() {
        let pool = Arc::new(AgentControllerPool::default());
        let registrations = [
            pool.register_agent_controller(agent_with_inbox("idle").agent_controller),
            pool.register_agent_controller(
                AgentControllerFixture::serving(
                    "embeddings",
                    InferenceMode::TextGenerationAndEmbeddings,
                    1,
                )
                .agent_controller,
            ),
        ];

        assert_eq!(
            registrations
                .iter()
                .map(AgentControllerRegistration::registered_agent_id)
                .collect::<Vec<_>>(),
            vec![Some("idle"), Some("embeddings")]
        );
        assert_eq!(
            pool.select_least_busy_with_capacity(
                one_slot_of(InferenceCapability::Embeddings),
                DispatchPosition::Arriving,
            )
            .map(|candidate| candidate.agent_controller.id.clone()),
            Some("embeddings".to_owned())
        );
    }

    #[test]
    fn refuses_a_second_agent_with_the_same_id() {
        let pool = Arc::new(AgentControllerPool::default());
        let registrations = [
            pool.register_agent_controller(agent_with_inbox("twin").agent_controller),
            pool.register_agent_controller(agent_with_inbox("twin").agent_controller),
        ];

        assert_eq!(
            registrations
                .iter()
                .map(AgentControllerRegistration::registered_agent_id)
                .collect::<Vec<_>>(),
            vec![Some("twin"), None]
        );
    }

    #[test]
    fn does_not_dispatch_a_capability_to_agents_whose_mode_lacks_it() {
        let pool = Arc::new(AgentControllerPool::default());
        let _registration = pool.register_agent_controller(
            AgentControllerFixture::serving(
                "completions",
                InferenceMode::TextGenerationAndEmbeddings,
                1,
            )
            .agent_controller,
        );

        assert_eq!(
            [
                InferenceCapability::TextGeneration,
                InferenceCapability::Decisions
            ]
            .map(|inference_capability| {
                pool.select_least_busy_with_capacity(
                    one_slot_of(inference_capability),
                    DispatchPosition::Arriving,
                )
                .map(|candidate| candidate.agent_controller.id.clone())
            }),
            [Some("completions".to_owned()), None]
        );
    }

    #[test]
    fn does_not_dispatch_a_demand_to_agents_with_fewer_free_slots_than_it_needs() {
        let pool = Arc::new(AgentControllerPool::default());
        let _registration = pool.register_agent_controller(
            AgentControllerFixture::serving(
                "decisions",
                InferenceMode::TextGenerationEmbeddingsAndDecisions,
                1,
            )
            .agent_controller,
        );

        assert!(
            pool.select_least_busy_with_capacity(
                DispatchRequirement {
                    inference_capability: InferenceCapability::Decisions,
                    slot_demand: SlotDemand::Exactly(DECISION_SLOTS_MINIMUM),
                },
                DispatchPosition::Arriving,
            )
            .is_none()
        );
    }

    #[test]
    fn a_dispatched_request_holds_its_reserved_slots_until_it_is_released() {
        let pool = Arc::new(AgentControllerPool::default());
        let agent_controller = AgentControllerFixture::serving(
            "decisions",
            InferenceMode::TextGenerationEmbeddingsAndDecisions,
            4,
        )
        .agent_controller;
        let _registration = pool.register_agent_controller(agent_controller.clone());

        let dispatched_agent = pool
            .take_least_busy_agent_controller(
                DispatchRequirement {
                    inference_capability: InferenceCapability::Decisions,
                    slot_demand: SlotDemand::Exactly(DECISION_SLOTS_MINIMUM),
                },
                DispatchPosition::Arriving,
            )
            .expect("an agent with free slots must be dispatched");

        assert_eq!(dispatched_agent.reserved_slots(), DECISION_SLOTS_MINIMUM);
        assert_eq!(
            agent_controller.slots_processing.get(),
            u64::from(DECISION_SLOTS_MINIMUM.get())
        );

        drop(dispatched_agent);

        assert_eq!(agent_controller.slots_processing.get(), 0);
    }

    #[test]
    fn counts_only_the_slots_of_serving_agents() {
        let pool = Arc::new(AgentControllerPool::default());
        let _registrations = [
            pool.register_agent_controller(agent_with_inbox("idle").agent_controller),
            pool.register_agent_controller(
                AgentControllerFixture::serving(
                    "serving",
                    InferenceMode::TextGenerationAndEmbeddings,
                    1,
                )
                .agent_controller,
            ),
        ];

        assert_eq!(pool.total_slots().slots_total, 1);
    }

    #[test]
    fn delivers_the_desired_state_to_connected_agents_past_a_disconnected_one() {
        let pool = Arc::new(AgentControllerPool::default());
        let disconnected_agent = agent_with_inbox("disconnected");
        let mut connected_agent = agent_with_inbox("connected");

        drop(disconnected_agent.agent_message_rx);

        let registrations = [
            pool.register_agent_controller(disconnected_agent.agent_controller),
            pool.register_agent_controller(connected_agent.agent_controller),
        ];

        assert!(registrations.iter().all(|registration| matches!(
            registration,
            AgentControllerRegistration::Registered(_)
        )));

        pool.set_desired_state(&AgentDesiredState::from(BalancerDesiredState::default()));

        assert!(matches!(
            connected_agent.agent_message_rx.try_recv(),
            Ok(AgentJsonRpcMessage::Notification(
                AgentJsonRpcNotification::SetState(set_state_params)
            )) if set_state_params.desired_state == AgentDesiredState::from(BalancerDesiredState::default())
        ));
    }

    #[test]
    fn a_later_request_takes_the_slots_beyond_what_waiting_requests_need() {
        let pool = Arc::new(AgentControllerPool::default());
        let _registration = pool.register_agent_controller(
            AgentControllerFixture::serving(
                "decisions",
                InferenceMode::TextGenerationEmbeddingsAndDecisions,
                4,
            )
            .agent_controller,
        );
        let _busy_slot = pool
            .take_least_busy_agent_controller(
                one_slot_of(InferenceCapability::TextGeneration),
                DispatchPosition::Arriving,
            )
            .expect("the agent must have a free slot");
        let _waiting_decision = pool.join_waiting_line(DispatchRequirement {
            inference_capability: InferenceCapability::Decisions,
            slot_demand: SlotDemand::Exactly(DECISION_SLOTS_MINIMUM),
        });

        assert_eq!(
            pool.take_least_busy_agent_controller(
                DispatchRequirement {
                    inference_capability: InferenceCapability::Embeddings,
                    slot_demand: SlotDemand::UpTo(4),
                },
                DispatchPosition::Arriving,
            )
            .map(|dispatched_agent| dispatched_agent.reserved_slots()),
            Some(NonZeroU16::MIN)
        );
    }

    #[test]
    fn a_waiting_request_holds_no_slots_on_agents_that_cannot_serve_it() {
        let pool = Arc::new(AgentControllerPool::default());
        let _registration = pool.register_agent_controller(
            AgentControllerFixture::serving(
                "completions",
                InferenceMode::TextGenerationAndEmbeddings,
                1,
            )
            .agent_controller,
        );
        let _waiting_decision = pool.join_waiting_line(DispatchRequirement {
            inference_capability: InferenceCapability::Decisions,
            slot_demand: SlotDemand::Exactly(DECISION_SLOTS_MINIMUM),
        });

        assert_eq!(
            pool.take_least_busy_agent_controller(
                one_slot_of(InferenceCapability::TextGeneration),
                DispatchPosition::Arriving,
            )
            .map(|dispatched_agent| dispatched_agent.reserved_slots()),
            Some(NonZeroU16::MIN)
        );
    }
}
