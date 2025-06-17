use anyhow::Result;
use cucumber::World;
use dashmap::DashMap;
use reqwest::Response;
use tokio::process::Child;

use super::llamacpp_instance::LlamaCppInstance;

#[derive(Debug, Default, World)]
pub struct BalancerWorld {
    pub agents: DashMap<String, Child>,
    pub balancer: Option<Child>,
    pub statsd: Option<Child>,
    pub prometheus: Option<Child>,
    pub last_llamacpp_port_offset: u16,
    pub llamas: DashMap<String, LlamaCppInstance>,
    pub requests: DashMap<String, Response>,
}

impl BalancerWorld {
    pub async fn cleanup(&mut self) {
        if let Some(mut balancer) = self.balancer.take() {
            if let Err(err) = balancer.kill().await {
                panic!("Failed to kill balancer: {err}");
            }
        }

        for mut agent in self.agents.iter_mut() {
            if let Err(err) = agent.value_mut().kill().await {
                panic!("Failed to kill agent {}: {}", agent.key(), err);
            }
        }

        if let Some(mut statsd) = self.statsd.take() {
            if let Err(err) = statsd.kill().await {
                panic!("Failed to kill statsd: {err}");
            }
        }

        if let Some(mut prometheus) = self.prometheus.take() {
            if let Err(err) = prometheus.kill().await {
                panic!("Failed to kill prometheus: {err}");
            }
        }

        for mut llama in self.llamas.iter_mut() {
            if let Err(err) = llama.value_mut().child.kill().await {
                panic!("Failed to kill llama {}: {}", llama.key(), err);
            }
        }
    }

    pub fn get_next_llamacpp_port(&mut self) -> u16 {
        let port = 8000 + self.last_llamacpp_port_offset;

        self.last_llamacpp_port_offset += 1;

        port
    }

    pub fn get_llamacpp_port(&self, llamacpp_name: &str) -> Result<u16> {
        if let Some(llama) = self.llamas.get(llamacpp_name) {
            Ok(llama.port)
        } else {
            Err(anyhow::anyhow!(
                "LlamaCpp instance {} not found",
                llamacpp_name
            ))
        }
    }
}
