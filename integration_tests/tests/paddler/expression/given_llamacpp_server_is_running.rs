use std::process::Stdio;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use cucumber::given;
use tempfile::NamedTempFile;
use tokio::process::Command;

use crate::llamacpp_instance::LlamaCppInstance;
use crate::paddler_world::PaddlerWorld;
use crate::retry_until_success::retry_until_success;

const MAX_ATTEMPTS: usize = 30;

async fn do_check(llamacpp_port: u16) -> Result<()> {
    let response = reqwest::get(format!("http://127.0.0.1:{llamacpp_port}/health")).await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Health check failed: Expected status 200, got {}",
            response.status()
        ));
    }

    let body = response.text().await?;

    if body.trim() != "OK" {
        return Err(anyhow!("Health check failed: Expected 'OK', got '{body}'"));
    }

    Ok(())
}

#[given(expr = "llama.cpp server {string} is running \\(has {int} slot(s)\\)")]
pub async fn given_agent_is_attached(
    world: &mut PaddlerWorld,
    llamacpp_name: String,
    available_slots: u16,
) -> Result<()> {
    if world.llamas.instances.contains_key(&llamacpp_name) {
        return Err(anyhow!(
            "Llama.cpp server {llamacpp_name} is already running"
        ));
    }

    let llamacpp_port = world.llamas.next_llamacpp_port();
    let log_file = NamedTempFile::new()?;

    world.llamas.instances.insert(
        llamacpp_name.clone(),
        LlamaCppInstance {
            child: Command::new("./tests/fixtures/llamacpp-server-mock.mjs")
                .arg("--completionResponseDelay=300")
                .arg(format!("--logFile={}", log_file.path().to_string_lossy()))
                .arg(format!("--name={llamacpp_name}"))
                .arg(format!("--port={llamacpp_port}"))
                .arg(format!("--slots={available_slots}"))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?,
            log_file,
            name: llamacpp_name.clone(),
            port: llamacpp_port,
        },
    );

    retry_until_success(
        || do_check(llamacpp_port),
        MAX_ATTEMPTS,
        Duration::from_millis(100),
        format!("Llama.cpp server {llamacpp_name} is still not running"),
    )
    .await
}
