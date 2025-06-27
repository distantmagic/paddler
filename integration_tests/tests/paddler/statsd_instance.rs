use tempfile::NamedTempFile;
use tokio::process::Child;

#[derive(Debug, Default)]
pub struct StatsdInstance {
    pub log_file: Option<NamedTempFile>,
    pub child: Option<Child>,
}

impl StatsdInstance {
    pub async fn kill(&mut self) {
        if let Some(child) = &mut self.child {
            if let Err(err) = child.kill().await {
                panic!("Failed to kill statsd: {}", err);
            }
        }
    }
}
