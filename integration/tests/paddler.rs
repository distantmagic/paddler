use cucumber::{given, then, when, World};
use paddler::{balancer::upstream_peer_pool::UpstreamPeerPool, errors::result::Result};
use serde_json::{json, Value};
use sysinfo::System;

use std::{env, fs::File, io::Write, net::SocketAddr, str::FromStr};
use tokio::process::Command;

use std::ffi::OsString;

use std::{result::Result as CoreResult, time::SystemTime};

use lazy_static::lazy_static;
use nix::sys::signal;
use nix::unistd::Pid;
use reqwest::Response;
use sysinfo::Process;
use tokio::process::Child;

lazy_static! {
    pub static ref PROMETHEUS_NAME: String =
        env::var("PROMETHEUS_NAME").expect("Failed to get env var PROMETHEUS_NAME");
    pub static ref STATSD_NAME: String =
        env::var("STATSD_NAME").expect("Failed to get env var STATSD_NAME");
    pub static ref LLAMACPP_NAME: String =
        env::var("LLAMACPP_NAME").expect("Failed to get env var LLAMACPP_NAME");
    pub static ref MODEL_NAME: String =
        env::var("MODEL_NAME").expect("Failed to get env var MODEL_NAME");
    pub static ref PADDLER_NAME: String =
        env::var("PADDLER_NAME").expect("Failed to get env var PADDLER_NAME");
}

#[derive(Debug, Default, cucumber::World)]
pub struct PaddlerWorld {
    pub balancer1: Option<Child>,
    pub agent1: Option<Child>,
    pub agent2: Option<Child>,
    pub supervisor1: Option<Child>,
    pub supervisor2: Option<Child>,
    pub system: Option<System>,
    pub llamacpp1: Option<Child>,
    pub llamacpp2: Option<Child>,
    pub statsd: Option<Child>,
    pub prometheus: Option<Child>,
    pub proxy_response: Vec<Option<CoreResult<Response, reqwest::Error>>>,
}

impl PaddlerWorld {
    pub async fn teardown(&mut self) -> Result<()> {
        let kill_process = async |process: &mut Option<Child>| {
            if let Some(child) = process {
                if let Some(pid) = child.id() {
                    let nix_pid = Pid::from_raw(pid as i32);

                    signal::kill(nix_pid, signal::Signal::SIGINT).unwrap();

                    let _ = child.wait().await.unwrap();
                }
            }
        };

        kill_process(&mut self.agent1).await;
        kill_process(&mut self.agent2).await;
        kill_process(&mut self.llamacpp1).await;
        kill_process(&mut self.llamacpp2).await;
        kill_process(&mut self.balancer1).await;
        kill_process(&mut self.statsd).await;
        kill_process(&mut self.prometheus).await;
        kill_process(&mut self.supervisor1).await;
        kill_process(&mut self.supervisor2).await;

        self.agent1 = None;
        self.agent2 = None;
        self.llamacpp1 = None;
        self.llamacpp2 = None;
        self.balancer1 = None;
        self.statsd = None;
        self.prometheus = None;
        self.supervisor1 = None;
        self.supervisor2 = None;

        Ok(())
    }
}

pub fn get_unix_time_from(secs: u64) -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() + secs,
        Err(err) => panic!("{:#?}", err),
    }
}

pub async fn kill_children(proc_id: Option<u32>) {
    let mut system = System::new_all();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let procs = get_children(proc_id, &system);

    for proc in procs {
        proc.kill();
        proc.wait();
    }
}

pub fn get_children(proc_id: Option<u32>, system: &System) -> Vec<&Process> {
    system
        .processes()
        .values()
        .filter(|process| {
            let parent_matches = match proc_id {
                Some(pid) => process.parent().map(|p| p.as_u32()) == Some(pid),
                None => true,
            };

            parent_matches
                && process.cmd().contains(&OsString::from("llama-server"))
                && !process.cmd().contains(&OsString::from("supervise"))
        })
        .collect()
}

#[given(
    expr = "{word} is running at {word}, {word} and reports metrics to {word} every {int} second(s)"
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
        Command::new(PADDLER_NAME.to_owned())
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

#[given(expr = "{word} is running at {word}, {word} and receives metrics from {word}")]
async fn statsd_is_running(
    world: &mut PaddlerWorld,
    _statsd_name: String,
    exporter_addr: String,
    management_addr: String,
) -> Result<()> {
    let mut cmd = Command::new(STATSD_NAME.to_owned());

    cmd.args([
        "--statsd.listen-udp",
        &exporter_addr,
        "--web.listen-address",
        &management_addr,
        "--log.level=debug",
    ]);

    world.statsd = Some(cmd.spawn()?);

    Ok(())
}

#[given(expr = "{word} is running at {word} and scrapes metrics from {word} every {int} second(s)")]
async fn prometheus_is_running(
    world: &mut PaddlerWorld,
    _prometheus_name: String,
    prometheus_addr: String,
    statsd_addr: String,
    _monitoring_interval: usize,
) -> Result<()> {
    let mut file = File::create("prometheus.yml")?;

    file.write(
        format!(
            "global:
    scrape_interval: 1s

scrape_configs:
  - job_name: 'paddler'
    static_configs:
    - targets: ['{}']",
            statsd_addr
        )
        .as_bytes(),
    )?;

    let mut cmd = Command::new(PROMETHEUS_NAME.to_owned());

    cmd.args(["--web.listen-address", &prometheus_addr]);

    world.prometheus = Some(cmd.spawn()?);

    Ok(())
}

#[given(
    expr = "{word} is running at {word} with {word} configuration stored on {word} and starts {word} at {int} with {int} slot(s) running"
)]
async fn supervisor_is_running(
    world: &mut PaddlerWorld,
    supervisor_name: String,
    supervisor_addr: String,
    driver_type: String,
    driver_addr: String,
    _llamacpp_name: String,
    llamacpp_addr: String,
    _slots: usize,
) -> Result<()> {
    let config_driver = match driver_type.as_str() {
        "file" => &format!(
            "{{\"type\": \"{}\", \"path\": \"{}\", \"name\": \"{}\"}}",
            driver_type, driver_addr, supervisor_name
        ),
        "etcd" => &format!(
            "{{\"type\": \"{}\", \"addr\": \"{}\", \"name\": \"{}\"}}",
            driver_type, driver_addr, supervisor_name
        ),
        _ => "",
    };

    let mut cmd = Command::new(PADDLER_NAME.to_owned());

    cmd.args([
        "supervise",
        "--supervisor-addr",
        &supervisor_addr,
        "--binary",
        &PADDLER_NAME,
        "--model",
        &MODEL_NAME,
        "--port",
        &llamacpp_addr,
        "--config-driver",
        config_driver,
    ]).spawn()?;

    match supervisor_name.as_str() {
        "supervisor-1" => world.supervisor1 = Some(cmd.spawn()?),
        "supervisor-2" => {
            world.supervisor2 = Some(cmd.spawn()?);
        }
        _ => (),
    }

    Ok(())
}

#[given(expr = "{word} is running at {word} with {int} slot(s)")]
async fn llamacpp_is_running(
    world: &mut PaddlerWorld,
    llamacpp_name: String,
    addr: String,
    slots: usize,
) -> Result<()> {
    let mut cmd = Command::new(LLAMACPP_NAME.to_owned());

    cmd.args([
        "-m",
        &MODEL_NAME,
        "-c",
        "2048",
        "-ngl",
        "2000",
        "-np",
        &slots.to_string(),
        "--slots",
        "--port",
        &addr.to_string(),
    ]);

    match llamacpp_name.as_str() {
        "llamacpp-1" => world.llamacpp1 = Some(cmd.spawn()?),
        "llamacpp-2" => world.llamacpp2 = Some(cmd.spawn()?),
        _ => (),
    }

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
        Command::new(PADDLER_NAME.to_owned())
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

    std::thread::sleep(std::time::Duration::from_secs(3));

    Ok(())
}

#[then(expr = "{word} in {word} must report that {word} is registered with {int} slots at {word}")]
async fn display_agent_slots(
    _world: &mut PaddlerWorld,
    _balancer_name: String,
    balancer_addr: String,
    agent_name: String,
    slots_idle: usize,
    llamacpp_addr: String,
) -> Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(15));

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

#[when(expr = r"{word} stops running and observing {word}, unregistered from {word}")]
async fn agent_is_not_running(world: &mut PaddlerWorld, agent_name: String) -> Result<()> {
    match agent_name.as_str() {
        "agent-1" => {
            if let Some(agent) = world.agent1.as_mut() {
                agent.kill().await?;
                agent.wait().await?;
                world.agent1 = None;
            }
        }
        "agent-2" => {
            if let Some(agent) = world.agent2.as_mut() {
                agent.kill().await?;
                agent.wait().await?;
                world.agent2 = None;
            }
        }
        _ => (),
    }

    Ok(())
}

#[then(expr = "{string} in {string} must report that {string} cannot fetch {string} in {string}")]
async fn agent_cannot_fetch_llamacpp(
    _world: &mut PaddlerWorld,
    _balancer_name: String,
    balancer_addr: String,
    agent_name: String,
    _llamacpp_name: String,
    llamacpp_addr: String,
) -> Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(15));

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

#[when(expr = r"{int} request(s) is/are proxied to {word} in {word}")]
async fn proxy_balancer(
    _world: &mut PaddlerWorld,
    requests: usize,
    _balancer_name: String,
    balancer_addr: String,
) -> Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(15));

    let client = reqwest::Client::new();

    let value = json!({
        "model": "qwen2_500m.gguf",
        "messages": [
            {
                "role": "user",
                "content": "List all prime numbers between 10,000 and 20,000,
                    verifying what are possible calculable primes by Lucas-Lehmer
                    test. Format as a numbered list with the verification proof
                    for each entry. And tell a story
                    about each number."
            }
        ]
    });

    let mut handles = vec![];

    for _ in 0..requests {
        let client = client.clone();
        let addr = balancer_addr.clone();
        let value = value.clone();
        handles.push(tokio::spawn(async move {
            client
                .post(format!("http://{}/chat/completions", addr))
                .json(&value)
                .send()
                .await
        }));
    }

    Ok(())
}

#[when(
    expr = r"{int} request(s) is/are proxied to {word} in {word} to change slots to {int} and port to {word}"
)]
async fn proxy_supervisor(
    _world: &mut PaddlerWorld,
    requests: usize,
    _supervisor_name: String,
    supervisor_addr: String,
    slots: usize,
    port: usize,
) -> Result<()> {
    let client = reqwest::Client::new();

    let value = json!(
    {
        "args": {
            "-m": MODEL_NAME.to_owned(),
            "--port": port,
            "binary": PADDLER_NAME.to_owned(),
            "-np": slots,
            "--slots": ""
        }
    }
    );

    let mut handles = vec![];

    for _ in 0..requests {
        let client = client.clone();
        let value = value.clone();
        let addr = supervisor_addr.clone();
        handles.push(tokio::spawn(async move {
            client
                .post(format!("http://{}/v1/params", addr))
                .json(&value)
                .send()
                .await
                .unwrap();
        }));
    }

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    Ok(())
}

#[then(
    expr = "{word} must tell {int} slot(s) is/are busy and {int} slot(s) is/are idle in {word} from {word} and {word}"
)]
async fn slot_is_busy(
    _world: &mut PaddlerWorld,
    _balancer_name: String,
    slots_busy: usize,
    idle_slots: usize,
    balancer_addr: String,
) -> Result<()> {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let response = serde_json::from_str::<UpstreamPeerPool>(
        reqwest::get(format!("http://{}/api/v1/agents", balancer_addr))
            .await?
            .text()
            .await?
            .as_str(),
    )?;

    let (slots_idle, slots_processing) = response.total_slots()?;

    assert_eq!(idle_slots, slots_idle);
    assert_eq!(slots_processing, slots_busy);

    Ok(())
}

#[when(expr = "{word} from {word} is killed")]
async fn kill_llamacpp(
    world: &mut PaddlerWorld,
    _llamacpp_name: String,
    supervisor_name: String,
) -> Result<()> {
    world.system = Some(System::new());

    match supervisor_name.as_str() {
        "supervisor-1" => {
            if let Some(supervisor) = &world.supervisor1 {
                let supervisor_pid = supervisor.id();
                kill_children(supervisor_pid).await;
            }
        }
        "supervisor-2" => {
            if let Some(supervisor) = &world.supervisor2 {
                let supervisor_pid = supervisor.id();
                kill_children(supervisor_pid).await;
            }
        }
        _ => (),
    };

    Ok(())
}

#[then(expr = r"{word} must return a(n) {word} response in {word}")]
async fn get_response(
    world: &mut PaddlerWorld,
    _balancer_name: String,
    _balancer_addr: String,
) -> Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(7));

    for response in &world.proxy_response {
        if let Some(response) = response {
            assert!(response.is_ok());
        }
    }

    world.proxy_response.clear();

    Ok(())
}

#[then(expr = "{word} in {word} must report that {word} does not exist")]
async fn agent_does_not_exist(
    _world: &mut PaddlerWorld,
    _balancer_name: String,
    balancer_addr: String,
    agent_name: String,
) -> Result<()> {
    std::thread::sleep(std::time::Duration::from_secs(15));

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

#[when(expr = r"{word} stops running")]
async fn llamacpp_is_not_running(world: &mut PaddlerWorld, llamacpp_name: String) -> Result<()> {
    match llamacpp_name.as_str() {
        "llamacpp-1" => {
            world.llamacpp1.as_mut().unwrap().kill().await?;
            world.llamacpp1.as_mut().unwrap().wait().await?;
        }
        "llamacpp-2" => {
            world.llamacpp2.as_mut().unwrap().kill().await?;
            world.llamacpp2.as_mut().unwrap().wait().await?;
        }
        _ => (),
    }

    Ok(())
}

#[then(expr = "{word} must tell {int} slot(s) is/are {word} at {word} from {word}")]
async fn report_metrics(
    _world: &mut PaddlerWorld,
    _statsd_name: String,
    slots: usize,
    state: String,
    prometheus_addr: String,
) -> Result<()> {
    let start = get_unix_time_from(0);
    let end = get_unix_time_from(15);
    let step = 5;

    std::thread::sleep(std::time::Duration::from_millis(2000));

    let response = reqwest::get(format!(
        "http://{}/api/v1/query?query=paddler_slots_{}&start={}&end={}&step={}",
        prometheus_addr, state, start, end, step
    ))
    .await?
    .text()
    .await?;

    let v: Value = serde_json::from_str(&response).unwrap();

    if let Some(metrics) = v.as_object() {
        if let Some(data) = metrics.get("data") {
            if let Some(result) = data.get("result") {
                if let Some(value) = result[0].get("value") {
                    assert_eq!(value[1].as_str(), Some(slots.to_string().as_str()))
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
pub async fn main() {
    PaddlerWorld::cucumber()
        .max_concurrent_scenarios(1)
        .fail_fast()
        // .retries(3)
        // .retry_after(std::time::Duration::from_secs(60))
        .fail_on_skipped()
        .after(|_feature, _rule, _scenario, _scenario_finished, world| {
            Box::pin(async move {
                world.unwrap().teardown().await.expect("Teardown Failed");
            })
        })
        .run("features")
        .await;
}
