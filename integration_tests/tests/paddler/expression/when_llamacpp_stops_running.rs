use std::time::Duration;

use anyhow::Result;
use cucumber::when;
use tokio::time::sleep;

use crate::paddler_world::PaddlerWorld;

const MAX_ATTEMPTS: usize = 3;

async fn do_check(llamacpp_port: u16) -> bool {
    match reqwest::get(format!("http://127.0.0.1:{llamacpp_port}/health")).await {
        Ok(response) => {
            if !response.status().is_success() {
                return false;
            }

            match response.text().await {
                Ok(body) => body.trim() == "OK",
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

#[when(expr = "llama.cpp server {string} stops running")]
pub async fn when_agent_detaches(world: &mut PaddlerWorld, llamacpp_name: String) -> Result<()> {
    if !world.llamas.instances.contains_key(&llamacpp_name) {
        return Err(anyhow::anyhow!(
            "Llama.cpp server {} is not running",
            llamacpp_name
        ));
    }

    let llamacpp_port = world.llamas.llamacpp_port(&llamacpp_name)?;

    world.llamas.kill(&llamacpp_name).await?;

    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        sleep(Duration::from_secs(1)).await;

        if !do_check(llamacpp_port).await {
            return Ok(());
        } else {
            eprintln!(
                "Attempt {}: llama.cpp at port {} is still alive.",
                attempts + 1,
                llamacpp_port
            );
        }

        attempts += 1;
    }

    Err(anyhow::anyhow!(
        "Llama.cpp server at port {} is still running after {} attempts",
        llamacpp_port,
        MAX_ATTEMPTS
    ))
}
