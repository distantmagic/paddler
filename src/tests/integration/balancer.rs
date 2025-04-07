// use core::result::Result as CoreResult;
// use cucumber::{given, then, when, World};
// use log::error;
// use reqwest::Response;
// use serde_json::json;
// use std::process::Child;

// use crate::{balancer::upstream_peer_pool::UpstreamPeerPool, errors::result::Result};

// use std::{env::current_dir, path::PathBuf, process::Command};

// #[derive(Debug, Default, cucumber::World)]
// struct PaddlerWorld {
//     pub agent1: Option<Child>,
//     pub agent2: Option<Child>,
//     pub llamacpp1: Option<Child>,
//     pub llamacpp2: Option<Child>,
//     pub balancer1: Option<Child>,
//     pub proxy_response: Vec<Option<CoreResult<Response, reqwest::Error>>>,
// }

// fn setup_project() -> Result<()> {
//     build_paddler()?;
//     download_llamacpp()?;
//     download_model()?;

//     Ok(())
// }

// fn download_llamacpp() -> Result<()> {
//     if cfg!(target_os = "windows") {
//         Command::new("winget")
//             .args(["install", "--id", "Git.Git", "-e", "--source winget"])
//             .status()?;
//     }
//     if cfg!(target_os = "macos") {
//         Command::new("xcode-select").arg("--install").status()?;
//     } else {
//         Command::new("sudo")
//             .args(["apt", "upgrade", "-y"])
//             .status()?;
//         Command::new("sudo")
//             .args(["apt", "install", "-y", "git"])
//             .status()?;
//         Command::new("sudo")
//             .args(["apt", "install", "-y", "git"])
//             .status()?;
//     };

//     build_llamacpp()?;

//     Ok(())
// }

// fn build_llamacpp() -> Result<()> {
//     Command::new("git")
//         .args(["clone", "https://github.com/ggml-org/llama.cpp.git"])
//         .status()?;

//     let previous_dir = current_dir()?;

//     std::env::set_current_dir("llama.cpp")?;

//     if cfg!(target_os = "windows") {
//         Command::new("cmake").args(["."]).status()?;
//         Command::new("cmake").args(["--build", "."]).status()?;
//     } else {
//         Command::new("cmake").args(["-B", "build"]).status()?;
//         Command::new("cmake")
//             .args(["--build", "build", "--config", "Release"])
//             .status()?;
//     };

//     std::env::set_current_dir(previous_dir)?;

//     Ok(())
// }

// fn download_model() -> Result<()> {
//     if cfg!(target_os = "windows") {
//         Command::new("powershell")
//             .args(["-Command", "Invoke-WebRequest -Uri 'https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf' -OutFile qwen2_500m.gguf"])
//             .status()?;
//     } else if cfg!(target_os = "macos") {
//         Command::new("curl")
//             .args(["-L", "-o", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf"])
//             .status()?;
//     } else {
//         Command::new("wget")
//             .args(["-O", "qwen2_500m.gguf", "https://huggingface.co/lmstudio-community/Qwen2-500M-Instruct-GGUF/resolve/main/Qwen2-500M-Instruct-IQ4_XS.gguf"])
//             .status()?;
//     };

//     Ok(())
// }

// fn build_paddler() -> Result<()> {
//     Command::new("make")
//         .args(["esbuild"])
//         .spawn()
//         .expect("Failed to run model");
//     Command::new("cargo")
//         .args(["build", "--features", "web_dashboard", "--release"])
//         .spawn()
//         .expect("Failed to run model");

//     Ok(())
// }

// fn start_llamacpp(port: usize, _name: &str) -> Result<Child> {
//     if !PathBuf::from("llama.cpp").exists() {
//         panic!("llama.cpp doesnt exist");
//     }

//     let mut command = if cfg!(target_os = "windows") {
//         if !PathBuf::from("llama.cpp/bin/Debug").exists() {
//             panic!("llama.cpp doesnt exist");
//         }

//         let mut cmd = Command::new("llama.cpp/bin/Debug/llama-server.exe");
//         cmd.args([
//             "-m",
//             "qwen2_500m.gguf",
//             "-c",
//             "2048",
//             "-ngl",
//             "2000",
//             "-np",
//             "4",
//             "-cb",
//             "--slots",
//             "--port",
//             &port.to_string(),
//         ]);
//         cmd
//     } else {
//         let mut cmd = Command::new("llama.cpp/build/bin/llama-server");
//         cmd.args([
//             "-m",
//             "qwen2_500m.gguf",
//             "-c",
//             "2048",
//             "-ngl",
//             "2000",
//             "-np",
//             "4",
//             "-cb",
//             "--slots",
//             "--port",
//             &port.to_string(),
//         ]);
//         cmd
//     };

//     Ok(command.spawn()?)
// }

// #[given(expr = r"{word} is running at {word}")]
// async fn start_balancer1(world: &mut PaddlerWorld) -> Result<()> {
//     if !PathBuf::from("target").exists() {
//         panic!("target doesnt exist");
//     }

//     world.balancer1 = Some(
//         Command::new("target/release/paddler")
//             .args([
//                 "balancer",
//                 "--management-addr",
//                 "localhost:8070",
//                 "--reverseproxy-addr",
//                 "0.0.0.0:8071",
//                 "--management-dashboard-enable",
//             ])
//             .spawn()
//             .expect("Failed to run balancer"),
//     );

//     Ok(())
// }

// #[given(expr = r"llamacpp-1 is running at {word} with {int} slot(s)")]
// async fn start_llamacpp1(world: &mut PaddlerWorld) -> Result<()> {
//     setup_project()?;
//     world.llamacpp1 = Some(start_llamacpp(8080, "agent1")?);

//     std::thread::sleep(std::time::Duration::from_secs(2));

//     Ok(())
// }

// #[given(expr = r"agent-1 is running and observing {word}, and registered at {word}")]
// async fn start_agent1(world: &mut PaddlerWorld) -> Result<()> {
//     world.agent1 = Some(
//         Command::new("target/release/paddler")
//             .args([
//                 "agent",
//                 "--local-llamacpp-addr",
//                 "0.0.0.0:8080",
//                 "--management-addr",
//                 "0.0.0.0:8070",
//                 "--name",
//                 "agent1",
//             ])
//             .spawn()
//             .expect("Failed to run balancer"),
//     );

//     Ok(())
// }

// #[given(expr = r"llamacpp-2 is running at {word} with {int} slot(s)")]
// async fn start_llamacpp2(world: &mut PaddlerWorld) -> Result<()> {
//     world.llamacpp2 = Some(start_llamacpp(8081, "agent2")?);

//     std::thread::sleep(std::time::Duration::from_secs(2));

//     Ok(())
// }

// #[given(expr = r"agent-2 is running and observing {word}, and registered at {word}")]
// async fn start_agent2(world: &mut PaddlerWorld) -> Result<()> {
//     world.agent2 = Some(
//         Command::new("target/release/paddler")
//             .args([
//                 "agent",
//                 "--local-llamacpp-addr",
//                 "0.0.0.0:8081",
//                 "--management-addr",
//                 "0.0.0.0':8070",
//                 "--name",
//                 "agent2",
//             ])
//             .spawn()
//             .expect("Failed to run balancer"),
//     );

//     Ok(())
// }

// // #[when(expr = r"first request is proxied to {word}")]
// // async fn first_request_to_balancer(world: &mut PaddlerWorld) -> Result<()> {
// //     let client = reqwest::Client::new();

// //     let value = json!({
// //         "model": "qwen2_500m.gguf",
// //         "messages": [
// //             {
// //                 "role": "user",
// //                 "content": "Write a limerick about python exceptions"
// //             }
// //         ]
// //     });

// //     std::thread::sleep(std::time::Duration::from_secs(2));

// //     world.proxy_response.push(Some(
// //         client
// //             .post("http://127.0.0.1:8071/v1/chat/completions")
// //             .body(value.to_string())
// //             .send()
// //             .await,
// //     ));

// //     Ok(())
// // }

// // #[then("{word} must tell {word} slot is busy")]
// // async fn slot_is_busy(_world: &mut PaddlerWorld) -> Result<()> {
// //     std::thread::sleep(std::time::Duration::from_secs(1));

// //     let mut response = serde_json::from_str::<UpstreamPeerPool>(
// //         reqwest::get("http://localhost:8070/api/v1/agents")
// //             .await?
// //             .text()
// //             .await?
// //             .as_str(),
// //     )?;

// //     let agents = response.agents.get_mut()?;

// //     let agent1 = agents
// //         .clone()
// //         .into_iter()
// //         .find(|agent1| agent1.agent_name == Some("agent1".to_string()));

// //     let agent2 = agents
// //         .into_iter()
// //         .find(|agent2| agent2.agent_name == Some("agent2".to_string()));

// //     if let (Some(agent1), Some(agent2)) = (agent1, agent2) {
// //         let idle_slots = agent1.slots_idle + agent2.slots_idle;
// //         let slots_processing = agent1.slots_processing + agent2.slots_processing;

// //         assert_eq!(idle_slots, 6);
// //         assert_eq!(slots_processing, 1);
// //         assert_eq!(agent1.error, None);
// //         assert_eq!(agent1.is_authorized, Some(true));
// //         assert_eq!(agent1.is_slots_endpoint_enabled, Some(true));
// //         assert_eq!(agent2.error, None);
// //         assert_eq!(agent2.is_authorized, Some(true));
// //         assert_eq!(agent2.is_slots_endpoint_enabled, Some(true));
// //     }

// //     Ok(())
// // }

// // #[when(expr = r"second request is proxied to {word}")]
// // async fn second_request_to_balancer(world: &mut PaddlerWorld) -> Result<()> {
// //     let client = reqwest::Client::new();

// //     let value = json!({
// //         "model": "qwen2_500m.gguf",
// //         "messages": [
// //             {
// //                 "role": "user",
// //                 "content": "Write a limerick about python exceptions"
// //             }
// //         ]
// //     });

// //     std::thread::sleep(std::time::Duration::from_secs(2));

// //     world.proxy_response.push(Some(
// //         client
// //             .post("http://127.0.0.1:8071/v1/chat/completions")
// //             .body(value.to_string())
// //             .send()
// //             .await,
// //     ));

// //     Ok(())
// // }

// // #[then("{word} must tell {word} slots are busy")]
// // async fn slots_are_busy(_world: &mut PaddlerWorld) -> Result<()> {
// //     std::thread::sleep(std::time::Duration::from_secs(1));

// //     let mut response = serde_json::from_str::<UpstreamPeerPool>(
// //         reqwest::get("http://localhost:8070/api/v1/agents")
// //             .await?
// //             .text()
// //             .await?
// //             .as_str(),
// //     )?;

// //     let agents = response.agents.get_mut()?;

// //     let agent1 = agents
// //         .clone()
// //         .into_iter()
// //         .find(|agent1| agent1.agent_name == Some("agent1".to_string()));

// //     let agent2 = agents
// //         .into_iter()
// //         .find(|agent2| agent2.agent_name == Some("agent2".to_string()));

// //     if let (Some(agent1), Some(agent2)) = (agent1, agent2) {
// //         let idle_slots = agent1.slots_idle + agent2.slots_idle;
// //         let slots_processing = agent1.slots_processing + agent2.slots_processing;

// //         assert_eq!(idle_slots, 5);
// //         assert_eq!(slots_processing, 2);
// //         assert_eq!(agent1.error, None);
// //         assert_eq!(agent1.is_authorized, Some(true));
// //         assert_eq!(agent1.is_slots_endpoint_enabled, Some(true));
// //         assert_eq!(agent2.error, None);
// //         assert_eq!(agent2.is_authorized, Some(true));
// //         assert_eq!(agent2.is_slots_endpoint_enabled, Some(true));
// //     }

// //     Ok(())
// // }

// // #[then("balancer-1 should return a successful response")]
// // async fn get_successful_response(world: &mut PaddlerWorld) -> Result<()> {
// //     std::thread::sleep(std::time::Duration::from_secs(7));

// //     for response in &world.proxy_response {
// //         if let Some(response) = response {
// //             assert!(response.is_ok());
// //         }
// //     }

// //     world.proxy_response.clear();

// //     Ok(())
// // }

// #[when(expr = r"{int} requests are proxied to {word}")]
// async fn proxy_requests(world: &mut PaddlerWorld) -> Result<()> {
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

//     for _ in 0..7 {
//         world.proxy_response.push(Some(
//             client
//                 .post("http://127.0.0.1:8071/v1/chat/completions")
//                 .body(value.to_string())
//                 .send()
//                 .await,
//         ));
//     }

//     Ok(())
// }

// #[then(expr = "{word} must tell {int} slots are busy")]
// async fn all_slots_are_busy(_world: &mut PaddlerWorld) -> Result<()> {
//     std::thread::sleep(std::time::Duration::from_secs(1));

//     let mut response = serde_json::from_str::<UpstreamPeerPool>(
//         reqwest::get("http://localhost:8070/api/v1/agents")
//             .await?
//             .text()
//             .await?
//             .as_str(),
//     )?;

//     let agents = response.agents.get_mut()?;

//     let agent1 = agents
//         .clone()
//         .into_iter()
//         .find(|agent1| agent1.agent_name == Some("agent1".to_string()));

//     let agent2 = agents
//         .into_iter()
//         .find(|agent2| agent2.agent_name == Some("agent2".to_string()));

//     if let (Some(agent1), Some(agent2)) = (agent1, agent2) {
//         let idle_slots = agent1.slots_idle + agent2.slots_idle;
//         let slots_processing = agent1.slots_processing + agent2.slots_processing;

//         assert_eq!(idle_slots, 0);
//         assert_eq!(slots_processing, 7);
//         assert_eq!(agent1.error, None);
//         assert_eq!(agent1.is_authorized, Some(true));
//         assert_eq!(agent1.is_slots_endpoint_enabled, Some(true));
//         assert_eq!(agent2.error, None);
//         assert_eq!(agent2.is_authorized, Some(true));
//         assert_eq!(agent2.is_slots_endpoint_enabled, Some(true));
//     }

//     Ok(())
// }

// #[when(expr = r"{int} request is proxied to {word}")]
// async fn proxy_request(world: &mut PaddlerWorld) -> Result<()> {
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

//     world.proxy_response.push(Some(
//         client
//             .post("http://127.0.0.1:8071/v1/chat/completions")
//             .body(value.to_string())
//             .send()
//             .await,
//     ));

//     Ok(())
// }

// #[then("balancer-1 should return an unsuccessful response")]
// async fn get_unsuccessful_response(world: &mut PaddlerWorld) -> Result<()> {
//     std::thread::sleep(std::time::Duration::from_secs(10));

//     for i in 0..7 {
//         if let Some(response) = &world.proxy_response[i] {
//             assert!(response.is_ok());
//             eprintln!("will print in {}", "tests")
//         }
//     }

//     if let Some(unsuccessful_response) = &world.proxy_response[7] {
//         assert!(unsuccessful_response.is_err());
//         eprintln!("will print in {}", "tests")
//     }

//     Ok(())
// }

// #[tokio::test]
// async fn run_cucumber_tests() {
//     PaddlerWorld::run("src/tests/integration/features/balancer.feature").await;
// }
