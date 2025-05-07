#[cfg(test)]
pub mod utils {
    use std::env;
    use std::ffi::OsString;
    use std::fs::File;
    use std::io::Write;

    use std::{result::Result as CoreResult, time::SystemTime};

    use nix::sys::signal;
    use nix::unistd::Pid;
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
            build().await?;

            Ok(())
        }
        
        pub async fn teardown(&mut self) -> Result<()> {
            let kill_process = async |process: &mut Option<Child>| {
                if let Some(child) = process {
                    if let Some(pid) = child.id() {
                        let nix_pid = Pid::from_raw(pid as i32);

                        if let Err(err) = signal::kill(nix_pid, signal::SIGINT) {
                            panic!("Failed to send SIGTERM: {err}");
                        }

                        if let Err(err) = child.wait().await {
                            panic!("Failed to wait for child process: {err}");
                        }
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

            Ok(())
        }
    }

    pub async fn build() -> Result<()> {
        Command::new("make")
            .args(["esbuild"])
            .spawn()?
            .wait()
            .await?;
        Command::new("cargo")
            .args(["build", "--features", "web_dashboard"])
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

        let binary_name = env::var("BINARY_NAME").expect("Failed to get env var BINARY_NAME");
        let model_name = env::var("MODEL_NAME").expect("Failed to get env var MODEL_NAME");

        let mut cmd = Command::new("target/release/paddler");

        Ok(cmd
            .args([
                "supervise",
                "--supervisor-addr",
                &supervisor_addr,
                "--binary",
                &binary_name,
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
        let statsd = env::var("STASTD_NAME").expect("Failed to get env var STATSD_NAME");

        let mut cmd = Command::new(statsd);

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

        let prometheus = env::var("PROMETHEUS").expect("Failed to get env var PROMETHEUS");

        let mut cmd = Command::new(prometheus);

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
