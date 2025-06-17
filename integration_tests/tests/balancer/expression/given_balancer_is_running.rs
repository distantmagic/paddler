use std::process::Stdio;

use anyhow::Result;
use cucumber::given;
use tokio::process::Command;

use crate::paddler_world::PaddlerWorld;

#[given("balancer is running")]
pub async fn given_balancer_is_running(world: &mut PaddlerWorld) -> Result<()> {
    if world.balancer.is_some() {
        return Err(anyhow::anyhow!("Balancer is already running"));
    }

    world.balancer = Some(
        Command::new("../target/debug/paddler")
            .arg("balancer")
            .arg("--management-addr=127.0.0.1:8095")
            .arg("--reverseproxy-addr=127.0.1:8096")
            .arg("--statsd-addr=localhost:9125")
            .arg("--statsd-reporting-interval=1")
            .arg("--request-timeout=3")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
    );

    Ok(())
}
