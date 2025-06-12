use std::process::Stdio;

use anyhow::Result;
use cucumber::given;
use tokio::process::Command;

use crate::balancer_world::BalancerWorld;

#[given("statsd is running")]
pub async fn given_statsd_is_running(world: &mut BalancerWorld) -> Result<()> {
    if world.statsd.is_some() {
        return Err(anyhow::anyhow!("Statsd is already running"));
    }

    world.statsd = Some(
        Command::new("./tests/fixtures/statsd-server-mock.mjs")
            .arg("managementPort=9102")
            .arg("exposePort=9102")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
    );

    Ok(())
}
