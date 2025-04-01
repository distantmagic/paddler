use cucumber::{given, then, when, World};

use crate::{
    balancer::upstream_peer_pool::UpstreamPeerPool,
    errors::result::Result,
};

use core::panic;
use std::{
    env::current_dir,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::Command,
};

#[derive(Debug, Default, cucumber::World)]
struct PaddlerWorld {}

fn setup_project() -> Result<()> {
    download_llamacpp()?;
    build_paddler()?;
    download_model()?;

    Ok(())
}

fn download_llamacpp() -> Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("winget")
            .args(["install", "--id", "Git.Git", "-e", "--source winget"])
            .status()?;
    }
    if cfg!(target_os = "macos") {
        Command::new("xcode-select").arg("--install").status()?;
    } else {
        Command::new("sudo")
            .args(["apt", "upgrade", "-y"])
            .status()?;
        Command::new("sudo")
            .args(["apt", "install", "-y", "git"])
            .status()?;
        Command::new("sudo")
            .args(["apt", "install", "-y", "git"])
            .status()?;
    };

    build_llamacpp()?;

    Ok(())
}

fn download_model() -> Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("Invoke-WebRequest")
            .args(["-Uri", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf?download=true", "-OutFile", "qwen2_500m.gguf"])
            .status()?;
    }
    if cfg!(target_os = "macos") {
        Command::new("curl").args(["-o", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf?download=true"]).status()?;
    } else {
        Command::new("wget")
            .args(["-O", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf?download=true"])
            .status()?;
    };
    Ok(())
}

fn build_llamacpp() -> Result<()> {
    Command::new("git")
        .args(["clone", "https://github.com/ggml-org/llama.cpp.git"])
        .status()?;

    let previous_dir = current_dir()?;

    std::env::set_current_dir("llama.cpp")?;

    if cfg!(target_os = "windows") {
        Command::new("cmake").args(["."]).status()?;
        Command::new("cmake").args(["--build", "."]).status()?;
    } else {
        Command::new("cmake").args(["-B", "build"]).status()?;
        Command::new("cmake")
            .args(["--build", "build", "--config", "Release"])
            .status()?;
    };

    std::env::set_current_dir(previous_dir)?;

    Ok(())
}

fn build_paddler() -> Result<()> {
    Command::new("cargo")
        .args(["build", "--features", "web_dashboard", "--release"])
        .spawn()
        .expect("Failed to run model");

    Ok(())
}

fn start_llamacpp(port: usize, _name: &str) -> Result<()> {
    let mut command = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("llama.cpp/bin/Debug/llama-server.exe");
        cmd.args([
            "-m",
            "qwen2_500m.gguf",
            "-c",
            "2048",
            "-ngl",
            "2000",
            "-np",
            "4",
            "-cb",
            "--slots",
            "--port",
            &port.to_string(),
        ]);
        cmd
    } else {
        let mut cmd = Command::new("llama.cpp/build/bin/llama-server");
        cmd.args([
            "-m",
            "qwen2_500m.gguf",
            "-c",
            "2048",
            "-ngl",
            "2000",
            "-np",
            "4",
            "-cb",
            "--slots",
            "--port",
            &port.to_string(),
        ]);
        cmd
    };

    command.spawn()?;

    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok(())
}

#[given(regex = r"llamacpp-1 is running at 0.0.0.0:8080 with 4 slots")]
async fn start_llamacpp1(_world: &mut PaddlerWorld) -> Result<()> {
    setup_project()?;
    start_llamacpp(8080, "agent1")?;

    Ok(())
}

#[given(regex = r"llamacpp-2 is running at 0.0.0.0:8081 with 3 slots")]
async fn start_llamacpp2(_world: &mut PaddlerWorld) -> Result<()> {
    start_llamacpp(8081, "agent2")?;

    Ok(())
}

#[given(regex = r"balancer-1 is running at 0.0.0.0:8070")]
async fn start_balancer1(_world: &mut PaddlerWorld) -> Result<()> {
    let _ = Command::new("target/release/paddler")
        .args([
            "balancer",
            "--management-addr",
            "0.0.0.0:8070",
            "--reverseproxy-addr",
            "0.0.0.0:8071",
            "--management-dashboard-enable",
        ])
        .spawn()
        .expect("Failed to run balancer");

    Ok(())
}

#[when(regex = r"agent-1 is running and observing llamacpp-1, and registered at balancer-1")]
async fn start_agent1(_world: &mut PaddlerWorld) -> Result<()> {
    let _ = Command::new("target/release/paddler")
        .args([
            "agent",
            "--external-llamacpp-addr",
            "0.0.0.0:8080",
            "--local-llamacpp-addr",
            "0.0.0.0:8080",
            "--management-addr",
            "0.0.0.0:8070",
            "--name",
            "agent1",
        ])
        .spawn()
        .expect("Failed to run balancer");

    Ok(())
}

#[then("balancer-1 should report that agent-1 is registered with 4 slots")]
async fn display_agent1_slots(_world: &mut PaddlerWorld) -> Result<()> {
    let mut response = serde_json::from_str::<UpstreamPeerPool>(
        reqwest::get("http://localhost:8070/api/v1/agents")
            .await?
            .text()
            .await?
            .as_str(),
    )?;
    let agents = response.agents.get_mut()?;

    let agent1 = agents
        .into_iter()
        .find(|agent1| agent1.agent_name == Some("agent1".to_string()));

    if let Some(agent1) = agent1 {
        assert_eq!(agent1.slots_idle, 4);
        assert_eq!(agent1.slots_processing, 0);
        assert_eq!(agent1.error, None);
        assert_eq!(
            agent1.external_llamacpp_addr,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080)
        );
        assert_eq!(agent1.is_authorized, Some(true));
        assert_eq!(agent1.is_slots_endpoint_enabled, Some(true));
    }

    Ok(())
}

#[when(regex = r"agent-2 is running and observing llamacpp-2, and registered at balancer-1")]
async fn start_agent2(_world: &mut PaddlerWorld) -> Result<()> {
    let _ = Command::new("target/release/paddler")
        .args([
            "agent",
            "--external-llamacpp-addr",
            "0.0.0.0:8081",
            "--local-llamacpp-addr",
            "0.0.0.0:8081",
            "--management-addr",
            "0.0.0.0:8070",
            "--name",
            "agent2",
        ])
        .spawn()
        .expect("Failed to run balancer");

    Ok(())
}

#[then("balancer-1 should report that agent-2 is registered with 3 slots")]
async fn display_agent2_slots(_world: &mut PaddlerWorld) -> Result<()> {
    let mut response = serde_json::from_str::<UpstreamPeerPool>(
        reqwest::get("http://localhost:8070/api/v1/agents")
            .await?
            .text()
            .await?
            .as_str(),
    )?;
    let agents = response.agents.get_mut()?;

    let agent1 = agents
        .into_iter()
        .find(|agent1| agent1.agent_name == Some("agent2".to_string()));

    if let Some(agent1) = agent1 {
        assert_eq!(agent1.slots_idle, 3);
        assert_eq!(agent1.slots_processing, 0);
        assert_eq!(agent1.error, None);
        assert_eq!(
            agent1.external_llamacpp_addr,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080)
        );
        assert_eq!(agent1.is_authorized, Some(true));
        assert_eq!(agent1.is_slots_endpoint_enabled, Some(true));
    }

    Ok(())
}

#[tokio::test]
async fn run_cucumber_tests() {
    PaddlerWorld::run("src/tests/integration/features/agent.feature").await;
}
