#[cfg(test)]
pub mod tests {
    use cucumber::{given, then, when, World};

    use crate::{
        balancer::upstream_peer_pool::UpstreamPeerPool,
        errors::result::Result,
        tests::integration::utils::utils::{start_llamacpp, PaddlerWorld},
    };

    use std::{net::SocketAddr, process::Command, str::FromStr};

    #[given(
        expr = "{word} is running at {word}, {word} and reports metrics to {word} every {int} second(s) in agent feature"
    )]
    async fn balancer_is_running(
        world: &mut PaddlerWorld,
        _balancer_name: String,
        management_addr: String,
        reveseproxy_addr: String,
        statsd_addr: String,
        reporting_interval: usize,
    ) -> Result<()> {
        world.balancer1 = Some(
            Command::new("target/release/paddler")
                .args([
                    "balancer",
                    "--management-addr",
                    &management_addr,
                    "--reverseproxy-addr",
                    &reveseproxy_addr,
                    "--statsd-addr",
                    &statsd_addr,
                    "--statsd-reporting-interval",
                    &reporting_interval.to_string(),
                    "--management-dashboard-enable",
                ])
                .spawn()
                .expect("Failed to run balancer"),
        );

        Ok(())
    }

    #[given(expr = "{word} is running at {word} with {int} slot(s) in agent feature")]
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
        expr = "{word} is running and observing {word} in {word}, and registered at {word} in {word} in agent feature"
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

    #[then(
        expr = "{word} in {word} must report that {word} is registered with {int} slots at {word} in agent feature"
    )]
    async fn display_agent_slots(
        _world: &mut PaddlerWorld,
        _balancer_name: String,
        balancer_addr: String,
        agent_name: String,
        slots_idle: usize,
        llamacpp_addr: String,
    ) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let mut response = serde_json::from_str::<UpstreamPeerPool>(
            &reqwest::get(format!("http://{}/api/v1/agents", balancer_addr))
                .await?
                .text()
                .await?,
        )?;

        let agents = response.agents.get_mut()?;

        let agent = agents
            .into_iter()
            .find(|agent| agent.agent_name == Some(agent_name.to_string()));

        if let Some(agent) = agent {
            assert_eq!(agent.slots_idle, slots_idle);
            assert_eq!(agent.slots_processing, 0);
            assert_eq!(agent.error, None);
            assert_eq!(
                agent.external_llamacpp_addr,
                SocketAddr::from_str(&llamacpp_addr)?
            );
            assert_eq!(agent.is_authorized, Some(true));
            assert_eq!(agent.is_slots_endpoint_enabled, Some(true));
        }

        Ok(())
    }

    #[when(
        expr = r"{word} stops running and observing {word}, unregistered from {word} in agent feature"
    )]
    async fn agent_is_not_running(world: &mut PaddlerWorld, agent_name: String) -> Result<()> {
        match agent_name.as_str() {
            "agent-1" => {
                if let Some(agent) = world.agent1.as_mut() {
                    agent.kill()?;
                    agent.wait()?;
                    world.agent1 = None;
                }
            }
            "agent-2" => {
                if let Some(agent) = world.agent2.as_mut() {
                    agent.kill()?;
                    agent.wait()?;
                    world.agent2 = None;
                }
            }
            _ => (),
        }

        Ok(())
    }

    #[then(expr = "{word} in {word} must report that {word} does not exist in agent feature")]
    async fn agent_does_not_exist(
        _world: &mut PaddlerWorld,
        _balancer_name: String,
        balancer_addr: String,
        agent_name: String,
    ) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(2));

        let mut response = serde_json::from_str::<UpstreamPeerPool>(
            &reqwest::get(format!("http://{}/api/v1/agents", balancer_addr))
                .await?
                .text()
                .await?,
        )?;
        let agents = response.agents.get_mut()?;

        let agent = agents
            .into_iter()
            .find(|agent| agent.agent_name == Some(agent_name.clone()));

        assert!(agent.is_none());

        Ok(())
    }

    #[when(expr = r"{word} stops running in agent feature")]
    async fn llamacpp_is_not_running(
        world: &mut PaddlerWorld,
        llamacpp_name: String,
    ) -> Result<()> {
        match llamacpp_name.as_str() {
            "llamacpp-1" => {
                world.llamacpp1.as_mut().unwrap().kill()?;
                world.llamacpp1.as_mut().unwrap().wait()?;
            }
            "llamacpp-2" => {
                world.llamacpp2.as_mut().unwrap().kill()?;
                world.llamacpp2.as_mut().unwrap().wait()?;
            }
            _ => (),
        }

        Ok(())
    }

    #[then(
        expr = "{word} in {word} must report that {word} cannot fetch {word} in {word} in agent feature"
    )]
    async fn agent_cannot_fetch_llamacpp(
        _world: &mut PaddlerWorld,
        _balancer_name: String,
        balancer_addr: String,
        agent_name: String,
        _llamacpp_name: String,
        llamacpp_addr: String,
    ) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_secs(10));

        let mut response = serde_json::from_str::<UpstreamPeerPool>(
            &reqwest::get(format!("http://{}/api/v1/agents", balancer_addr))
                .await?
                .text()
                .await?,
        )?;
        let agents = response.agents.get_mut()?;

        let agent = agents
            .into_iter()
            .find(|agent| agent.agent_name == Some(agent_name.clone()));

        if let Some(agent) = agent {
            assert!(agent.error.is_some());
            assert_eq!(
                agent.error,
                Some(format!(
                    "Request error: error sending request for url (http://{}/slots)",
                    llamacpp_addr
                ))
            );
            assert_eq!(agent.is_authorized, None);
            assert_eq!(agent.is_slots_endpoint_enabled, None);
        }

        Ok(())
    }

    pub async fn run_agent_tests() {
        PaddlerWorld::cucumber()
            .max_concurrent_scenarios(1)
            .fail_fast()
            // .retries(3)
            // .retry_after(std::time::Duration::from_secs(60))
            .fail_on_skipped()
            .after(|_feature, _rule, _scenario, _scenario_finished, world| {
                Box::pin(async move {
                    if let Some(world) = world {
                        world.teardown().await.expect("Teardown Failed");
                    }
                })
            })
            .run("src/tests/integration/features/agent.feature")
            .await;
    }
}
