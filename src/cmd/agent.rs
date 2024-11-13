use actix::Actor;
use log::error;
use tokio::time;

use crate::agent::{agent::Agent, state_reporter::StateReporter};
use crate::errors::result::Result;
use crate::llamacpp::llamacpp_client::LlamacppClient;

pub async fn handle(
    local_llamacpp_addr: &url::Url,
    local_llamacpp_api_key: &Option<String>,
    management_addr: &url::Url,
    name: &Option<String>,
) -> Result<()> {
    let state_reporter_addr = StateReporter::new(management_addr.clone())?.start();
    let llamacpp_client =
        LlamacppClient::new(local_llamacpp_addr.clone(), local_llamacpp_api_key.clone())?;
    let agent = Agent::new(llamacpp_client, name.clone());
    let mut interval = time::interval(time::Duration::from_secs(1));

    loop {
        interval.tick().await;

        if let Err(err) = agent.observe_and_report(&state_reporter_addr).await {
            error!("Error: {}", err);
        }
    }
}
