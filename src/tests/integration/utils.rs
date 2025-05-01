#[cfg(test)]
pub mod utils {
    use std::ffi::OsString;
    use std::fs::File;
    use std::io::Write;

    use std::{env::current_dir, result::Result as CoreResult, time::SystemTime};

    use log::error;
    use reqwest::Response;
    use sysinfo::{Process, System};
    use tokio::process::{Child, Command};

    use crate::errors::result::Result;

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
        pub async fn setup() -> Result<()> {
            // download_llamacpp().await?;
            // download_model().await?;
            // build_paddler().await?;

            Ok(())
        }

        pub async fn teardown(&mut self) -> Result<()> {
            let mut errors = Vec::new();

            let mut kill_process = async |process: &mut Option<Child>| {
                if let Some(p) = process {
                    match p.kill().await {
                        Ok(_) => {
                            if let Err(e) = p.wait().await {
                                errors.push(format!("Failed to wait for process: {}", e));
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Failed to kill process: {}", e));
                        }
                    }
                    *process = None;
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

            kill_children(None).await;

            Ok(())
        }
    }

    async fn download_llamacpp() -> Result<()> {
        Command::new("git")
            .args(["clone", "https://github.com/ggml-org/llama.cpp.git"])
            .spawn()?
            .wait()
            .await?;

        let previous_dir = current_dir()?;

        std::env::set_current_dir("llama.cpp")?;

        Command::new("git")
            .args([
                "reset",
                "--hard",
                "f52d59d771dc231fc2ac39adacf157ddefc97730",
            ])
            .spawn()?
            .wait()
            .await?;
        Command::new("git")
            .args(["clean", "-df", "f52d59d771dc231fc2ac39adacf157ddefc97730"])
            .spawn()?
            .wait()
            .await?;

        match std::env::consts::OS {
            "windows" => {
                Command::new("cmake").args(["."]).spawn()?.wait().await?;
                Command::new("cmake")
                    .args(["--build", "."])
                    .spawn()?
                    .wait()
                    .await?;
            }
            _ => {
                Command::new("cmake")
                    .args(["-B", "build"])
                    .spawn()?
                    .wait()
                    .await?;
                Command::new("cmake")
                    .args(["--build", "build", "--config", "Release"])
                    .spawn()?
                    .wait()
                    .await?;
            }
        };

        std::env::set_current_dir(previous_dir)?;

        Ok(())
    }

    pub async fn download_model() -> Result<()> {
        match std::env::consts::OS {
            "windows" => {
                Command::new("powershell")
                    .args(["-Command", "Invoke-WebRequest -Uri 'https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf' -OutFile qwen2_500m.gguf"])
                    .spawn()?
                    .wait().await?;
            }
            "macos" => {
                Command::new("curl")
                    .args(["-L", "-o", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf"])
                    .spawn()?
                    .wait().await?;
            }
            "linux" => {
                Command::new("wget")
                    .args(["-O", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf"])
                    .spawn()?
                    .wait().await?;
            }
            _ => (),
        };

        Ok(())
    }

    pub async fn build_paddler() -> Result<()> {
        Command::new("make")
            .args(["esbuild"])
            .spawn()?
            .wait()
            .await?;
        Command::new("cargo")
            .args(["build", "--features", "web_dashboard", "--release"])
            .spawn()?
            .wait()
            .await?;

        Ok(())
    }

    pub async fn start_llamacpp(port: String, slots: usize) -> Result<Child> {
        let mut cmd = match std::env::consts::OS {
            "windows" => {
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
                    "--slots",
                    "--port",
                    &port.to_string(),
                ]);
                cmd
            }
            _ => {
                let mut cmd = Command::new("llama.cpp/build/bin/llama-server");
                cmd.args([
                    "-m",
                    "qwen2_500m.gguf",
                    "-c",
                    "2048",
                    "-ngl",
                    "2000",
                    "-nocb",
                    "-np",
                    &slots.to_string(),
                    "--slots",
                    "--port",
                    &port.to_string(),
                ]);
                cmd
            }
        };

        Ok(cmd.spawn()?)
    }

    pub async fn start_supervisor(
        supervisor_name: String,
        supervisor_addr: String,
        driver_type: String,
        driver_addr: String,
        llamacpp_addr: String,
        model_name: String,
    ) -> Result<Child> {
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

        let mut cmd = Command::new("target/release/paddler");

        Ok(cmd
            .args([
                "supervise",
                "--supervisor-addr",
                &supervisor_addr,
                "--binary",
                "llama-server",
                "--model",
                &model_name,
                "--port",
                &llamacpp_addr,
                "--config-driver",
                config_driver,
            ])
            .kill_on_drop(true)
            .process_group(0)
            .spawn()?)
    }

    pub async fn start_statsd(management_addr: String, exporter_addr: String) -> Result<Child> {
        let mut cmd = match cfg!(target_os = "windows") {
            true => Command::new("./statsd_exporter.exe"),
            false => Command::new("statsd_exporter"),
        };

        cmd.args([
            "--statsd.listen-udp",
            &exporter_addr,
            "--web.listen-address",
            &management_addr,
            "--log.level=debug",
        ]);

        Ok(cmd.spawn()?)
    }

    pub async fn start_prometheus(
        prometheus_addr: String,
        management_addr: String,
    ) -> Result<Child> {
        let mut file = File::create("prometheus.yml")?;

        file.write(
            format!(
                "global:
    scrape_interval: 1s

scrape_configs:
  - job_name: 'paddler'
    static_configs:
    - targets: ['{}']",
                management_addr
            )
            .as_bytes(),
        )?;

        let mut cmd = match cfg!(target_os = "windows") {
            true => Command::new("./prometheus.exe"),
            false => Command::new("prometheus"),
        };

        cmd.args(["--web.listen-address", &prometheus_addr]);

        Ok(cmd.spawn()?)
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
}
