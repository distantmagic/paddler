use tokio::process::Child;

#[derive(Debug)]
pub struct SupervisorInstance {
    pub child: Child,
    pub llamacpp_listen_port: u16,
}
