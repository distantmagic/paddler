use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use parking_lot::Mutex;

use paddler_messaging::agent_runtime_status::AgentRuntimeStatus;

use crate::agent_controller_pool_updates::AgentControllerPoolUpdates;
use crate::dispatch_position::DispatchPosition;
use crate::dispatch_requirement::DispatchRequirement;

pub struct DispatchWaitingLine {
    next_ticket: AtomicU64,
    updates: Arc<AgentControllerPoolUpdates>,
    waiting_requirements: Mutex<BTreeMap<u64, DispatchRequirement>>,
}

impl DispatchWaitingLine {
    #[must_use]
    pub const fn new(updates: Arc<AgentControllerPoolUpdates>) -> Self {
        Self {
            next_ticket: AtomicU64::new(0),
            updates,
            waiting_requirements: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn join(&self, dispatch_requirement: DispatchRequirement) -> u64 {
        let mut waiting_requirements = self.waiting_requirements.lock();
        let ticket = self.next_ticket.fetch_add(1, Ordering::Relaxed);

        waiting_requirements.insert(ticket, dispatch_requirement);

        ticket
    }

    pub fn leave(&self, ticket: u64) {
        self.waiting_requirements.lock().remove(&ticket);
        self.updates.signal_waiting_line_change();
    }

    #[must_use]
    pub fn slots_held_ahead_of(
        &self,
        dispatch_position: DispatchPosition,
        agent_runtime_status: &AgentRuntimeStatus,
    ) -> u64 {
        self.waiting_requirements
            .lock()
            .iter()
            .filter(|(ticket, _dispatch_requirement)| dispatch_position.waits_behind(**ticket))
            .map(|(_ticket, dispatch_requirement)| {
                dispatch_requirement.slots_held_while_waiting(agent_runtime_status)
            })
            .sum()
    }
}
