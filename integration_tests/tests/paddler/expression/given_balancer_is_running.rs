use std::process::Stdio;

use anyhow::Result;
use cucumber::given;
use tokio::process::Command;

use crate::paddler_world::PaddlerWorld;

#[given(expr = "balancer is running \\({int} max request(s)\\)")]
pub async fn given_balancer_is_running(world: &mut PaddlerWorld, max_requests: u16) -> Result<()> {
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
            .arg("--request-timeout=0")
            .arg(format!("--max-requests={max_requests}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
    );

    Ok(())
}
