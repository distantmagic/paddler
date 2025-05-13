use cucumber::{given, then, when, World};
use paddler::{balancer::upstream_peer_pool::UpstreamPeerPool, errors::result::Result};
use serde_json::{json, Value};
use sysinfo::System;

use std::sync::{Arc, Mutex};
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
    pub balancer: Option<Child>,
    pub agents: Vec<Option<Child>>,
    pub supervisors: Vec<Option<Child>>,
    pub system: Option<System>,
    pub llamacpps: Vec<Option<Child>>,
    pub statsd: Option<Child>,
    pub prometheus: Option<Child>,
    pub proxy_response: Vec<CoreResult<Response, reqwest::Error>>,
}

impl PaddlerWorld {
    pub async fn teardown(&mut self) -> Result<()> {
        Self::kill_all_processes(self).await;
        Self::reset_all_processes(self);

        Ok(())
    }

    async fn kill_all_processes(&mut self) {
        kill_process(&mut self.balancer).await;
        kill_process(&mut self.statsd).await;
        kill_process(&mut self.prometheus).await;

        if let Some(agent) = self.agents.get_mut(0) {
            kill_process(agent).await;
        }
        if let Some(agent) = self.agents.get_mut(1) {
            kill_process(agent).await;
        }
        if let Some(agent) = self.agents.get_mut(2) {
            kill_process(agent).await;
        }
        if let Some(llamacpp) = self.llamacpps.get_mut(0) {
            kill_process(llamacpp).await;
        }
        if let Some(llamacpp) = self.llamacpps.get_mut(1) {
            kill_process(llamacpp).await;
        }
        if let Some(supervisor) = self.supervisors.get_mut(0) {
            kill_process(supervisor).await;
        }
        if let Some(supervisor) = self.supervisors.get_mut(1) {
            kill_process(supervisor).await;
        }
    }

    fn reset_all_processes(&mut self) {
        if let Some(agent) = self.agents.get_mut(0) {
            *agent = None;
        }
        if let Some(agent) = self.agents.get_mut(1) {
            *agent = None;
        }
        if let Some(llamacpp) = self.llamacpps.get_mut(0) {
            *llamacpp = None;
        }
        if let Some(llamacpp) = self.llamacpps.get_mut(1) {
            *llamacpp = None;
        }
        if let Some(supervisor) = self.supervisors.get_mut(0) {
            *supervisor = None;
        }
        if let Some(supervisor) = self.supervisors.get_mut(1) {
            *supervisor = None;
        }

        self.balancer = None;
        self.statsd = None;
        self.prometheus = None;
    }
}

async fn kill_process(process: &mut Option<Child>) {
    if let Some(child) = process {
        if let Some(pid) = child.id() {
            let nix_pid = Pid::from_raw(pid as i32);

            signal::kill(nix_pid, signal::Signal::SIGINT).unwrap();

            let _ = child.wait().await.unwrap();
        }
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
    world.balancer = Some(
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
            .kill_on_drop(true)
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

    world.statsd = Some(cmd.kill_on_drop(true).spawn()?);

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

    world.prometheus = Some(cmd.kill_on_drop(true).spawn()?);

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

    let child = cmd
        .args([
            "supervise",
            "--supervisor-addr",
            &supervisor_addr,
            "--binary",
            &LLAMACPP_NAME,
            "--model",
            &MODEL_NAME,
            "--port",
            &llamacpp_addr,
            "--config-driver",
            config_driver,
        ])
        .kill_on_drop(true)
        .spawn()?;

    match supervisor_name.as_str() {
        "supervisor-1" => world.supervisors.push(Some(child)),
        "supervisor-2" => world.supervisors.push(Some(child)),
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

    let child = cmd
        .args([
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
        ])
        .spawn()?;

    match llamacpp_name.as_str() {
        "llamacpp-1" => world.llamacpps.push(Some(child)),
        "llamacpp-2" => world.llamacpps.push(Some(child)),
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
    let mut cmd = Command::new(PADDLER_NAME.to_owned());

    let child = cmd
        .args([
            "agent",
            "--local-llamacpp-addr",
            &llamacpp_addr,
            "--management-addr",
            &balancer_addr,
            "--name",
            &agent_name,
        ])
        .kill_on_drop(true)
        .spawn()?;

    match agent_name.as_str() {
        "agent-1" => world.agents.push(Some(child)),
        "agent-2" => world.agents.push(Some(child)),
        _ => (),
    }

    tokio::time::sleep(std::time::Duration::from_secs(4)).await;

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
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

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
            if let Some(agent) = world.agents.get_mut(0) {
                kill_process(agent).await;
                world.agents.insert(0, None);
            }
        }
        "agent-2" => {
            if let Some(agent) = world.agents.get_mut(1) {
                kill_process(agent).await;
                world.agents.push(None);
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
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

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
                "content": "List all prime numbers between 10,000 and 20,000"
            }
        ]
    });

    let (tx, rx) = tokio::sync::mpsc::channel(requests);

    tokio::spawn(async move {
        let mut rx = rx;
        while let Some(response) = rx.recv().await {
            world.proxy_response.push(response);
        }
    });

    for _ in 0..requests {
        let client = client.clone();
        let addr = balancer_addr.clone();
        let value = value.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            let result = client
                .post(format!("http://{}/chat/completions", addr))
                .json(&value)
                .send()
                .await;
            
            let _ = tx.send(result).await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
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
            "binary": LLAMACPP_NAME.to_owned(),
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
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

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
            if let Some(Some(supervisor)) = &world.supervisors.get(0) {
                let supervisor_pid = supervisor.id();
                kill_children(supervisor_pid).await;
            }
        }
        "supervisor-2" => {
            if let Some(Some(supervisor)) = &world.supervisors.get(1) {
                let supervisor_pid = supervisor.id();
                kill_children(supervisor_pid).await;
            }
        }
        _ => (),
    };

    Ok(())
}

#[then(expr = r"{word} must return a(n) {word} response in {word}")]
async fn successful_response(
    world: &mut PaddlerWorld,
    _balancer_name: String,
    response: String,
    _balancer_addr: String,
) -> Result<()> {
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

    match response.as_str() {
        "successful" => {
            for response in &world.proxy_response {
                assert!(response.as_ref().is_ok());
            }
        }
        "unsuccessful" => {
            for response in &world.proxy_response {
                assert!(response.as_ref().is_err());
            }
        }
        _ => (),
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
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;

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
            world.llamacpps[0].as_mut().unwrap().kill().await?;
            world.llamacpps[0].as_mut().unwrap().wait().await?;
        }
        "llamacpp-2" => {
            world.llamacpps[1].as_mut().unwrap().kill().await?;
            world.llamacpps[1].as_mut().unwrap().wait().await?;
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
    let end = get_unix_time_from(30);
    let step = 5;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

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
        // .retries(2)
        // .retry_after(std::time::Duration::from_secs(60))
        .fail_on_skipped()
        .after(|_feature, _rule, _scenario, _scenario_finished, world| {
            // match scenario_finished {
            //     cucumber::event::ScenarioFinished::StepFailed(_regex, _step , _error) => {
            //         panic!("{:#?}", world);
            //     }
            //     _ => ()
            // }
            Box::pin(async move {
                world.unwrap().teardown().await.expect("Teardown Failed");
            })
        })
        .run_and_exit("features")
        .await;
}
