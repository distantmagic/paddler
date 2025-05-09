pub mod utils{
    use std::env;
    use std::ffi::OsString;
    use std::fs::File;
    use std::io::Write;
    
    use std::{result::Result as CoreResult, time::SystemTime};
    
    use lazy_static::lazy_static;
    use nix::sys::signal;
    use nix::unistd::Pid;
    use paddler::errors::result::Result;
    use reqwest::Response;
    use sysinfo::{Process, System};
    use tokio::process::{Child, Command};
    
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
}
