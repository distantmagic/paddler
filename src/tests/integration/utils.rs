use std::{
    env::current_dir,
    fs::File,
    io::Write,
    process::{Child, Command},
    result::Result as CoreResult,
};

use reqwest::Response;

use crate::errors::result::Result;

#[derive(Debug, Default, cucumber::World)]
pub struct PaddlerWorld {
    pub agent1: Option<Child>,
    pub agent2: Option<Child>,
    pub llamacpp1: Option<Child>,
    pub llamacpp2: Option<Child>,
    pub balancer1: Option<Child>,
    pub statsd: Option<Child>,
    pub proxy_response: Vec<Option<CoreResult<Response, reqwest::Error>>>,
}

impl PaddlerWorld {
    pub fn setup(&mut self) -> Result<()> {
        build_paddler()?;
        download_llamacpp()?;
        download_model()?;
        download_node()?;
        download_statsd()?;

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
        kill_process(&mut self.statsd);

        Ok(())
    }
}

pub fn download_llamacpp() -> Result<()> {
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

fn build_llamacpp() -> Result<()> {
    Command::new("git")
        .args(["clone", "https://github.com/ggml-org/llama.cpp.git"])
        .status()?;

    let previous_dir = current_dir()?;

    std::env::set_current_dir("llama.cpp")?;

    Command::new("git")
        .args([
            "reset",
            "--hard",
            "f52d59d771dc231fc2ac39adacf157ddefc97730",
        ])
        .status()?;
    Command::new("git")
        .args(["clean", "-df", "f52d59d771dc231fc2ac39adacf157ddefc97730"])
        .status()?;

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

fn download_statsd() -> Result<()> {
    Command::new("git")
        .args(["clone", "https://github.com/statsd/statsd.git"])
        .status()?;

    Ok(())
}

fn download_node() -> Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("winget")
            .args(["install", "Schniz.fnm"])
            .status()?;
    } else {
        Command::new("curl")
            .args(["-o-", "https://fnm.vercel.app/install | bash"])
            .status()?;
    };

    Command::new("fnm").args(["install", "22"]).status()?;

    Ok(())
}

pub fn download_model() -> Result<()> {
    if cfg!(target_os = "windows") {
        Command::new("powershell")
            .args(["-Command", "Invoke-WebRequest -Uri 'https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf' -OutFile qwen2_500m.gguf"])
            .status()?;
    } else if cfg!(target_os = "macos") {
        Command::new("curl")
            .args(["-L", "-o", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf"])
            .status()?;
    } else {
        Command::new("wget")
            .args(["-O", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf"])
            .status()?;
    };

    Ok(())
}

pub fn build_paddler() -> Result<()> {
    Command::new("make")
        .args(["esbuild"])
        .spawn()
        .expect("Failed to run model");
    Command::new("cargo")
        .args(["build", "--features", "web_dashboard", "--release"])
        .spawn()
        .expect("Failed to run model");

    Ok(())
}

pub fn start_llamacpp(port: String, slots: usize) -> Result<Child> {
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
            &slots.to_string(),
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
            &slots.to_string(),
            "-cb",
            "--slots",
            "--port",
            &port.to_string(),
        ]);
        cmd
    };

    Ok(command.spawn()?)
}

pub fn start_statsd(host: String, metrics_port: String, management_port: String) -> Result<Child> {
    let previous_dir = current_dir()?;

    std::env::set_current_dir("statsd")?;

    let mut file = File::create("config.js")?;

        file.write(
            format!(
                r#"{{
      address: "{}"
    , port: {}
    , mgmt_port: {}
    , mgmt_address: "{}"
    }}"#,
                host, metrics_port, management_port, host
            )
            .as_bytes(),
        )?;

    let mut command = Command::new("node");

    command.args(["stats.js", "config.js"]);

    std::env::set_current_dir(previous_dir)?;

    Ok(command.spawn()?)
}
