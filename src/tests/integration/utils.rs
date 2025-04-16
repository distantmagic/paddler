#[cfg(test)]
pub mod utils {
    use std::{
        env::current_dir,
        fs::File,
        io::Write,
        process::{Child, Command},
        result::Result as CoreResult,
        time::SystemTime,
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
        pub prometheus: Option<Child>,
        pub proxy_response: Vec<Option<CoreResult<Response, reqwest::Error>>>,
    }

    impl PaddlerWorld {
        pub fn setup(&mut self) -> Result<()> {
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
            kill_process(&mut self.statsd);
            kill_process(&mut self.prometheus);

            Ok(())
        }
    }

    fn download_llamacpp() -> Result<()> {
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

        match std::env::consts::OS {
            "windows" => {
                Command::new("cmake").args(["."]).status()?;
                Command::new("cmake").args(["--build", "."]).status()?;
            }
            _ => {
                Command::new("cmake").args(["-B", "build"]).status()?;
                Command::new("cmake")
                    .args(["--build", "build", "--config", "Release"])
                    .status()?;
            }
        };

        std::env::set_current_dir(previous_dir)?;

        Ok(())
    }

    pub fn download_model() -> Result<()> {
        match std::env::consts::OS {
            "windows" => {
                Command::new("powershell")
            .args(["-Command", "Invoke-WebRequest -Uri 'https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf' -OutFile qwen2_500m.gguf"])
            .status()?;
            }
            "macos" => {
                Command::new("curl")
            .args(["-L", "-o", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf"])
            .status()?;
            }
            "linux" => {
                Command::new("wget")
            .args(["-O", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf"])
            .status()?;
            }
            _ => (),
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
                    "-cb",
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
                    "-np",
                    &slots.to_string(),
                    "-cb",
                    "--slots",
                    "--port",
                    &port.to_string(),
                ]);
                cmd
            }
        };

        Ok(cmd.spawn()?)
    }

    pub fn start_statsd(management_addr: String, exporter_addr: String) -> Result<Child> {
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

    pub fn start_prometheus(prometheus_addr: String, management_addr: String) -> Result<Child> {
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
}
