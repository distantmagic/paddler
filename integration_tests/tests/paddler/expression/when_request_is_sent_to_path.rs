use anyhow::Result;
use cucumber::when;

use crate::REVERSE_PROXY_PORT;
use crate::paddler_world::PaddlerWorld;

#[when(expr = "request {string} is sent to {string}")]
pub async fn when_request_is_sent_to_path(
    world: &mut PaddlerWorld,
    name: String,
    path: String,
) -> Result<()> {
    let request = world
        .request_builder
        .get(
            &name,
            format!("http://127.0.0.1:{REVERSE_PROXY_PORT}{path}"),
        )
        .header("X-Request-Name", name.clone());

    let response = request.send().await?;

    world.responses.insert(name, response);

    Ok(())
}
