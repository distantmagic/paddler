use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use cucumber::when;

use crate::paddler_world::PaddlerWorld;
use crate::retry_until_success::retry_until_success;

const MAX_ATTEMPTS: usize = 30;

async fn do_check(llamacpp_port: u16) -> Result<()> {
    match reqwest::get(format!("http://127.0.0.1:{llamacpp_port}/health")).await {
        Ok(_) => Err(anyhow!(
            "Health check got to the server; expected llama.cpp server to be stopped"
        )),
        Err(_) => Ok(()),
    }
}

#[when(expr = "llama.cpp server {string} stops running")]
pub async fn when_llamacpp_stops_running(
    world: &mut PaddlerWorld,
    llamacpp_name: String,
) -> Result<()> {
    if !world.llamas.instances.contains_key(&llamacpp_name) {
        return Err(anyhow!("Llama.cpp server {llamacpp_name} is not running"));
    }

    let llamacpp_port = world.llamas.llamacpp_port(&llamacpp_name)?;

    world.llamas.kill(&llamacpp_name).await?;

    retry_until_success(
        || do_check(llamacpp_port),
        MAX_ATTEMPTS,
        Duration::from_millis(100),
        format!("Llama.cpp server at port {llamacpp_port} is still running"),
    )
    .await
}
