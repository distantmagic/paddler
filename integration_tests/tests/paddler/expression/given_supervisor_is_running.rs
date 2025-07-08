use std::process::Stdio;

use anyhow::Result;
use anyhow::anyhow;
use cucumber::given;
use tokio::process::Command;

use crate::BALANCER_PORT;
use crate::MOCK_LLAMACPP_SERVER_PATH;
use crate::paddler_world::PaddlerWorld;
use crate::supervisor_instance::SupervisorInstance;

#[given(expr = "supervisor {string} is running")]
pub async fn given_supervisor_is_running(
    world: &mut PaddlerWorld,
    supervisor_name: String,
) -> Result<()> {
    if world.supervisors.instances.contains_key(&supervisor_name) {
        return Err(anyhow!("Agent {supervisor_name} is already running"));
    }

    let llamacpp_listen_port = world.llamas.next_llamacpp_port();
    let child = Command::new("../target/debug/paddler")
        .arg("supervisor")
        .arg(format!("--name={supervisor_name}"))
        .arg(format!(
            "--llamacpp-listen-addr=127.0.0.1:{llamacpp_listen_port}"
        ))
        .arg(format!(
            "--llamacpp-server-bin-path={MOCK_LLAMACPP_SERVER_PATH}"
        ))
        .arg(format!("--management-addr=127.0.0.1:{BALANCER_PORT}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    world.supervisors.instances.insert(
        supervisor_name.clone(),
        SupervisorInstance {
            child,
            llamacpp_listen_port,
        },
    );

    Ok(())
}
