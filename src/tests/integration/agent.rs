use cucumber::{given, then, when, World};
use std::process::Child;

use crate::{balancer::upstream_peer_pool::UpstreamPeerPool, errors::result::Result};

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    process::Command,
};

use super::utils::{build_paddler, download_llamacpp, download_model, start_llamacpp};

#[derive(Debug, Default, World)]
struct PaddlerWorld {
    pub agent1: Option<Child>,
    pub agent2: Option<Child>,
    pub llamacpp1: Option<Child>,
    pub llamacpp2: Option<Child>,
    pub balancer1: Option<Child>,
}

impl PaddlerWorld {
    pub fn setup(&mut self) -> Result<()> {
        build_paddler()?;
        download_llamacpp()?;
        download_model()?;

        Ok(())
    }

    pub async fn teardown(&mut self) -> Result<()> {
        let mut errors = Vec::new();

        let mut kill_process = |process: &mut Option<Child>| {
            if let Some(p) = process {
                if let Err(err) = p.kill() {
                    errors.push(format!("Failed to kill: {}", err));
                }
                *process = None;
            }
        };

        kill_process(&mut self.agent1);
        kill_process(&mut self.agent2);
        kill_process(&mut self.llamacpp1);
        kill_process(&mut self.llamacpp2);
        kill_process(&mut self.balancer1);

        Ok(())
    }
}

#[given(expr = "{word} is running at {word} and {word}")]
async fn balancer_is_running(
    world: &mut PaddlerWorld,
    _balancer_name: String,
    management_addr: String,
    reveseproxy_addr: String,
) -> Result<()> {
    world.balancer1 = Some(
        Command::new("target/release/paddler")
            .args([
                "balancer",
                "--management-addr",
                &management_addr,
                "--reverseproxy-addr",
                &reveseproxy_addr,
                "--management-dashboard-enable",
            ])
            .spawn()
            .expect("Failed to run balancer"),
    );

    Ok(())
}

#[given(expr = "{word} is running at {word} with {int} slot(s)")]
async fn llamacpp_is_running(
    world: &mut PaddlerWorld,
    llamacpp_name: String,
    addr: String,
    slots: usize,
) -> Result<()> {
    match llamacpp_name.as_str() {
        "llamacpp-1" => world.llamacpp1 = Some(start_llamacpp(addr, slots)?),
        "llamacpp-2" => world.llamacpp2 = Some(start_llamacpp(addr, slots)?),
        _ => (),
    }

    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(())
}

#[when(
    expr = "{word} is running and observing {word} in {word}, and registered at {word} in {word}"
)]
async fn agent_is_running(
    world: &mut PaddlerWorld,
    agent_name: String,
    _llamacpp_name: String,
    llamacpp_addr: String,
    _balancer_name: String,
    balancer_addr: String,
) -> Result<()> {
    let process = Some(
        Command::new("target/release/paddler")
            .args([
                "agent",
                "--local-llamacpp-addr",
                &llamacpp_addr,
                "--management-addr",
                &balancer_addr,
                "--name",
                &agent_name,
            ])
            .spawn()
            .expect("Failed to run balancer"),
    );

    match agent_name.as_str() {
        "agent-1" => world.agent1 = process,
        "agent-2" => world.agent2 = process,
        _ => (),
    }

    Ok(())
}

#[then(expr = "{word} must report that {word} is registered with {int} slot(s)")]
async fn display_agent_slots(
    _world: &mut PaddlerWorld,
    _balancer_name: String,
    agent_name: String,
    slots: usize,
) -> Result<()> {
    let mut response = serde_json::from_str::<UpstreamPeerPool>(
        &reqwest::get("http://localhost:8070/api/v1/agents")
            .await?
            .text()
            .await?,
    )?;
    let agents = response.agents.get_mut()?;

    let agent = agents
        .into_iter()
        .find(|agent| agent.agent_name == Some(agent_name.to_string()));

    if let Some(agent) = agent {
        assert_eq!(agent.slots_idle, slots);
        assert_eq!(agent.slots_processing, 0);
        assert_eq!(agent.error, None);
        assert_eq!(
            agent.external_llamacpp_addr,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080)
        );
        assert_eq!(agent.is_authorized, Some(true));
        assert_eq!(agent.is_slots_endpoint_enabled, Some(true));
    }

    Ok(())
}

// #[when(expr = r"{word} stops running and observing {word}, deregistered from {word}")]
// async fn agent_is_not_running(world: &mut PaddlerWorld, agent_name: String) -> Result<()> {
//     match agent_name.as_str() {
//         "agent-1" => {
//             if let Some(agent) = world.agent1.as_mut() {
//                 agent.kill()?;
//             }
//         }
//         "agent-2" => {
//             if let Some(agent) = world.agent2.as_mut() {
//                 agent.kill()?;
//             }
//         }
//         _ => (),
//     }

//     Ok(())
// }

// #[then(expr = "{word} in {word} must report that {word} does not exist")]
// async fn agent_does_not_exist(
//     _world: &mut PaddlerWorld,
//     _balancer_name: String,
//     balancer_addr: String,
//     agent_name: String,
// ) -> Result<()> {
//     let mut response = serde_json::from_str::<UpstreamPeerPool>(
//         &reqwest::get(format!("http://{}/api/v1/agents", balancer_addr))
//             .await?
//             .text()
//             .await?,
//     )?;
//     let agents = response.agents.get_mut()?;

//     let agent = agents
//         .into_iter()
//         .find(|agent| agent.agent_name == Some(agent_name.clone()));

//     assert!(agent.is_none());

//     Ok(())
// }

// #[when(expr = r"{word} stops running")]
// async fn llamacpp_is_not_running(world: &mut PaddlerWorld, llamacpp_name: String) -> Result<()> {
//     match llamacpp_name.as_str() {
//         "llamacpp-1" => world.llamacpp1.as_mut().unwrap().kill()?,
//         "llamacpp-2" => world.llamacpp2.as_mut().unwrap().kill()?,
//         _ => (),
//     }

//     Ok(())
// }

// #[then(expr = "{word} in {word} must report that {word} cannot fetch {word} in {word}")]
// async fn agent_cannot_fetch_llamacpp(
//     _world: &mut PaddlerWorld,
//     _balancer_name: String,
//     balancer_addr: String,
//     agent_name: String,
//     _llamacpp_name: String,
//     llamacpp_addr: String,
// ) -> Result<()> {
//     let mut response = serde_json::from_str::<UpstreamPeerPool>(
//         &reqwest::get(format!("http://{}/api/v1/agents", balancer_addr))
//             .await?
//             .text()
//             .await?,
//     )?;
//     let agents = response.agents.get_mut()?;

//     let agent = agents
//         .into_iter()
//         .find(|agent| agent.agent_name == Some(agent_name.clone()));

//     if let Some(agent) = agent {
//         assert!(agent.error.is_some());
//         assert_eq!(
//             agent.error,
//             Some(format!(
//                 "Request error: error sending request for url (http://{}/slots)",
//                 llamacpp_addr
//             ))
//         );
//         assert_eq!(agent.is_authorized, None);
//         assert_eq!(agent.is_slots_endpoint_enabled, None);
//     }

//     Ok(())
// }

#[tokio::test]
async fn run_cucumber_tests() {
    PaddlerWorld::cucumber()
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.setup().expect("Setup failed");
            })
        })
        .after(|_feature, _rule, _scenario, _scenario_finished, world| {
            Box::pin(async move {
                if let Some(world) = world {
                    world.teardown().await.expect("Teardown Failed");
                }
            })
        })
        .run("src/tests/integration/features/agent.feature")
        .await;

    PaddlerWorld::run("src/tests/integration/features/agent.feature").await;
}
