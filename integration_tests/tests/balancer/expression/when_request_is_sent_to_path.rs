use anyhow::Result;
use cucumber::when;

use crate::balancer_world::BalancerWorld;

#[when(expr = "request {string} is sent to {string}")]
pub async fn when_request_is_sent_to_path(
    world: &mut BalancerWorld,
    name: String,
    path: String,
) -> Result<()> {
    let response = reqwest::get(format!("http://127.0.0.1:8096{path}")).await?;

    world.requests.insert(name, response);

    Ok(())
}
