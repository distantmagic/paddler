use std::process::Stdio;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use cucumber::given;
use tokio::process::Command;

use crate::paddler_world::PaddlerWorld;
use crate::retry_until_success::retry_until_success;

const MAX_ATTEMPTS: usize = 50;

async fn do_check(statsd_port: u16) -> Result<()> {
    let response = reqwest::get(format!("http://127.0.0.1:{statsd_port}/health")).await?;

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

#[given("statsd is running")]
pub async fn given_statsd_is_running(world: &mut PaddlerWorld) -> Result<()> {
    if world.statsd.child.is_some() {
        return Err(anyhow::anyhow!("Statsd is already running"));
    }

    let statsd_port = 9102;

    world.statsd.child = Some(
        Command::new("./tests/fixtures/statsd-server-mock.mjs")
            .arg("--managementPort=9125")
            .arg(format!("--exposePort={statsd_port}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
    );

    retry_until_success(
        || do_check(statsd_port),
        MAX_ATTEMPTS,
        Duration::from_millis(100),
        format!("Statsd server at port {statsd_port} is stil not responding"),
    )
    .await
}
