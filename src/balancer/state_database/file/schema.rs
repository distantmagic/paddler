use serde::Deserialize;
use serde::Serialize;

use crate::balancer_desired_state::BalancerDesiredState;

#[derive(Default, Deserialize, Serialize)]
pub struct Schema {
    pub balancer_desired_state: BalancerDesiredState,
}
