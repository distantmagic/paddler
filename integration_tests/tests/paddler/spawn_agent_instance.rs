use std::process::Stdio;

use anyhow::Result;
use tokio::process::Child;
use tokio::process::Command;

use crate::BALANCER_PORT;

pub fn spawn_agent_instance(agent_name: String, local_llamacpp_port: u16) -> Result<Child> {
    Ok(Command::new("../target/debug/paddler")
        .arg("agent")
        .arg(format!("--name={agent_name}"))
        .arg(format!(
            "--local-llamacpp-addr=127.0.0.1:{local_llamacpp_port}"
        ))
        .arg(format!("--management-addr=127.0.0.1:{BALANCER_PORT}"))
        .arg("--monitoring-interval=500")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?)
}
