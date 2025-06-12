use std::process::Stdio;

use anyhow::Result;
use cucumber::given;
use tokio::process::Command;

use crate::balancer_world::BalancerWorld;

#[given(expr = "prometheus is running \\(scrapes every {int} second(s)\\)")]
pub async fn given_prometheus_is_running(world: &mut BalancerWorld, seconds: u16) -> Result<()> {
    if world.prometheus.is_some() {
        return Err(anyhow::anyhow!("Prometheus is already running"));
    }

    world.statsd = Some(
        Command::new("./tests/fixtures/prometheus-server-mock.mjs")
            .arg("target=9102")
            .arg("managementPort=9090")
            .arg(format!("scrapeInterval={seconds}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
    );

    Ok(())
}
