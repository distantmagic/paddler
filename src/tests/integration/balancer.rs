use cucumber::{given, then, when, World};
use serde_json::{json, Value};

use crate::errors::result::Result;

use std::{io::Read, net::TcpStream, process::Command};

use super::utils::{start_llamacpp, start_statsd, PaddlerWorld};

#[given(expr = "{word} is running at {word}, {word} and reports metrics to {word}")]
async fn balancer_is_running(
    world: &mut PaddlerWorld,
    _balancer_name: String,
    management_addr: String,
    reveseproxy_addr: String,
    statsd_addr: String,
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
                "--management-dashboard-enable",
            ])
            .spawn()
            .expect("Failed to run balancer"),
    );

    Ok(())
}

#[given(expr = "{word} is running at {word} in {int}, {int} and receives metrics from {word}")]
async fn statsd_is_running(
    world: &mut PaddlerWorld,
    _statsd_name: String,
    host: String,
    metrics_port: String,
    management_port: String,
) -> Result<()> {
    world.statsd = Some(start_statsd(host, metrics_port, management_port)?);

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

    Ok(())
}

#[given(
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

// #[when(expr = r"first request is proxied to {word} in {word}")]
// async fn request_to_balancer(
//     world: &mut PaddlerWorld,
//     _balancer_name: String,
//     balancer_addr: String,
// ) -> Result<()> {
//     let client = reqwest::Client::new();

//     let value = json!({
//         "model": "qwen2_500m.gguf",
//         "messages": [
//             {
//                 "role": "user",
//                 "content": "Write a limerick about python exceptions"
//             }
//         ]
//     });

//     std::thread::sleep(std::time::Duration::from_secs(10));

//     world.proxy_response.push(Some(
//         client
//             .post(format!("http://{}/v1/chat/completions", balancer_addr))
//             .body(value.to_string())
//             .send()
//             .await,
//     ));

//     Ok(())
// }

// #[then(
//     expr = "{word} must tell {int} slot(s) is/are busy and {int} slot(s) is/are idle in {word} from {word} and {word}"
// )]
// async fn slot_is_busy(
//     _world: &mut PaddlerWorld,
//     _balancer_name: String,
//     slots_busy: usize,
//     slots_idle: usize,
//     balancer_addr: String,
//     agent1_name: String,
//     agent2_name: String,
// ) -> Result<()> {
//     let mut response = serde_json::from_str::<UpstreamPeerPool>(
//         reqwest::get(format!("http://{}/api/v1/agents", balancer_addr))
//             .await?
//             .text()
//             .await?
//             .as_str(),
//     )?;

//     std::thread::sleep(std::time::Duration::from_secs(2));

//     let agents = response.agents.get_mut()?;

//     let agent1 = agents
//         .clone()
//         .into_iter()
//         .find(|agent1| agent1.agent_name == Some(agent1_name.clone()));

//     let agent2 = agents
//         .into_iter()
//         .find(|agent2| agent2.agent_name == Some(agent2_name.clone()));

//     if let (Some(agent1), Some(agent2)) = (agent1, agent2) {
//         let idle_slots = agent1.slots_idle + agent2.slots_idle;
//         let slots_processing = agent1.slots_processing + agent2.slots_processing;

//         assert_eq!(idle_slots, slots_idle);
//         assert_eq!(slots_processing, slots_busy);
//         assert_eq!(agent1.error, None);
//         assert_eq!(agent1.is_authorized, Some(true));
//         assert_eq!(agent1.is_slots_endpoint_enabled, Some(true));
//         assert_eq!(agent2.error, None);
//         assert_eq!(agent2.is_authorized, Some(true));
//         assert_eq!(agent2.is_slots_endpoint_enabled, Some(true));
//     }

//     Ok(())
// }

// #[when(expr = r"second request is proxied to {word} in {word}")]
// async fn second_request_to_balancer(
//     world: &mut PaddlerWorld,
//     _balancer_name: String,
//     balancer_addr: String,
// ) -> Result<()> {
//     let client = reqwest::Client::new();

//     let value = json!({
//         "model": "qwen2_500m.gguf",
//         "messages": [
//             {
//                 "role": "user",
//                 "content": "Write a limerick about python exceptions"
//             }
//         ]
//     });

//     std::thread::sleep(std::time::Duration::from_secs(2));

//     world.proxy_response.push(Some(
//         client
//             .post(format!("http://{}/v1/chat/completions", balancer_addr))
//             .body(value.to_string())
//             .send()
//             .await,
//     ));

//     Ok(())
// }

// #[then(expr = "{word} should return a successful response in {word}")]
// async fn get_successful_response(
//     world: &mut PaddlerWorld,
//     _balancer_name: String,
//     _balancer_addr: String,
// ) -> Result<()> {
//     std::thread::sleep(std::time::Duration::from_secs(7));

//     for response in &world.proxy_response {
//         if let Some(response) = response {
//             assert!(response.is_ok());
//         }
//     }

//     world.proxy_response.clear();

//     Ok(())
// }

#[when(expr = r"{int} request(s) is/are proxied to {word} in {word}")]
async fn proxy_requests(
    world: &mut PaddlerWorld,
    requests: usize,
    _balancer_name: String,
    balancer_addr: String,
) -> Result<()> {
    let client = reqwest::Client::new();

    let value = json!({
        "model": "qwen2_500m.gguf",
        "messages": [
            {
                "role": "user",
                "content": "Write a limerick about python exceptions"
            }
        ]
    });

    for _ in 0..requests {
        world.proxy_response.push(Some(
            client
                .post(format!("http://{}/v1/chat/completions", balancer_addr))
                .body(value.to_string())
                .send()
                .await,
        ));
    }

    Ok(())
}

#[then(
    expr = "{word} must tell {int} slot(s) is/are busy and {int} slot(s) is/are idle at {word} in {int}"
)]
async fn report_metrics(
    _world: &mut PaddlerWorld,
    _statsd_name: String,
    _slots_busy: usize,
    _slots_idle: usize,
    statsd_host: String,
    statsd_port: usize,
) -> Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut stream = TcpStream::connect(format!("{}:{}", statsd_host, statsd_port))?;

    let mut buffer = Vec::new();

    stream.read_to_end(&mut buffer)?;

    let response = String::from_utf8(buffer).unwrap();
    let metrics: Value = serde_json::from_str(&response)?;

    assert_eq!(metrics, "a");

    Ok(())
}

#[tokio::test]
async fn run_cucumber_tests() {
    PaddlerWorld::cucumber()
        .max_concurrent_scenarios(1)
        .before(|_feature, _rule, _scenario, world| {
            Box::pin(async move {
                world.setup().expect("Teardown Failed");
            })
        })
        .after(|_feature, _rule, _scenario, _scenario_finished, world| {
            Box::pin(async move {
                if let Some(world) = world {
                    world.teardown().await.expect("Teardown Failed");
                }
            })
        })
        .run("src/tests/integration/features/balancer.feature")
        .await;
}
