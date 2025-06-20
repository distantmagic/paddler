use core::panic;
use std::process::Stdio;
use std::time::Duration;

use anyhow::Result;
use cucumber::given;
use tempfile::NamedTempFile;
use tokio::process::Command;
use tokio::time::sleep;

use crate::llamacpp_instance::LlamaCppInstance;
use crate::paddler_world::PaddlerWorld;

const MAX_ATTEMPTS: usize = 3;

async fn do_check(llamacpp_port: u16) -> Result<()> {
    let response = reqwest::get(format!("http://127.0.0.1:{llamacpp_port}/health")).await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Health check failed: Expected status 200, got {}",
            response.status()
        ));
    }

    let body = response.text().await?;

    if body.trim() != "OK" {
        return Err(anyhow::anyhow!(
            "Health check failed: Expected 'OK', got '{}'",
            body
        ));
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
        return Err(anyhow::anyhow!(
            "Llama.cpp server {} is already running",
            llamacpp_name
        ));
    }

    let llamacpp_port = world.llamas.next_llamacpp_port();
    let log_file = NamedTempFile::new()?;

    world.llamas.instances.insert(
        llamacpp_name.clone(),
        LlamaCppInstance {
            child: Command::new("./tests/fixtures/llamacpp-server-mock.mjs")
                .arg("--completionResponseDelay=1")
                .arg(format!("--logFile={}", log_file.path().to_string_lossy()))
                .arg(format!("--name={llamacpp_name}"))
                .arg(format!("--port={llamacpp_port}"))
                .arg(format!("--slots={available_slots}"))
                // .stdout(Stdio::null())
                // .stderr(Stdio::null())
                .spawn()?,
            log_file,
            name: llamacpp_name,
            port: llamacpp_port,
        },
    );

    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        sleep(Duration::from_secs(1)).await;

        match do_check(llamacpp_port).await {
            Ok(_) => return Ok(()),
            Err(err) => eprintln!(
                "Attempt {}: llama.cpp is not ready yet: {err}",
                attempts + 1
            ),
        };

        attempts += 1;
    }

    Err(anyhow::anyhow!(
        "Llama.cpp server at port {} did not start after {} attempts",
        llamacpp_port,
        MAX_ATTEMPTS
    ))
}
