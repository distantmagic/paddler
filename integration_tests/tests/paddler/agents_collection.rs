use dashmap::DashMap;
use tokio::process::Child;

#[derive(Debug, Default)]
pub struct AgentsCollection {
    pub instances: DashMap<String, Child>,
}

impl AgentsCollection {
    pub async fn cleanup(&mut self) {
        for mut agent in self.instances.iter_mut() {
            if let Err(err) = agent.value_mut().kill().await {
                panic!("Failed to kill agent {}: {}", agent.key(), err);
            }
        }

        self.instances.clear();
    }

    pub async fn kill(&self, llamacpp_name: String) {
        if let Some(mut agent) = self.instances.get_mut(&llamacpp_name) {
            if let Err(err) = agent.value_mut().kill().await {
                eprintln!("Failed to kill agent {}: {}", llamacpp_name, err);
            }
            
            self.instances.remove(&llamacpp_name);
        }
    }
}
