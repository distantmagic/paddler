use tokio::process::Child;

#[derive(Debug)]
pub struct LlamaCppInstance {
    pub child: Child,
    pub port: u16,
}
