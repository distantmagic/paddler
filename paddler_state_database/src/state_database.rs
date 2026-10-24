use paddler_messaging::balancer_desired_state::BalancerDesiredState;

use crate::file::File;
use crate::memory::Memory;
use crate::state_database_error::StateDatabaseError;
use crate::stored_balancer_desired_state::StoredBalancerDesiredState;

pub enum StateDatabase {
    File(File),
    Memory(Memory),
}

impl StateDatabase {
    pub async fn read_balancer_desired_state(
        &self,
    ) -> Result<StoredBalancerDesiredState, StateDatabaseError> {
        match self {
            Self::File(file) => file.read_balancer_desired_state().await,
            Self::Memory(memory) => Ok(memory.read_balancer_desired_state()),
        }
    }

    pub async fn store_balancer_desired_state(
        &self,
        balancer_desired_state: &BalancerDesiredState,
    ) -> Result<(), StateDatabaseError> {
        match self {
            Self::File(file) => {
                file.store_balancer_desired_state(balancer_desired_state)
                    .await
            }
            Self::Memory(memory) => {
                memory.store_balancer_desired_state(balancer_desired_state);

                Ok(())
            }
        }
    }
}
