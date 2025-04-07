use std::{
    env::current_dir,
    process::{Child, Command},
};

use crate::errors::result::Result;

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

    if cfg!(target_os = "windows") {
        Command::new("cmake").args(["."]).status()?;
        Command::new("cmake").args(["--build", "."]).status()?;
    } else {
        Command::new("cmake").args(["-B", " "]).status()?;
        Command::new("cmake")
            .args(["--build", "build", "--config", "Release"])
            .status()?;
    };

    std::env::set_current_dir(previous_dir)?;

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
