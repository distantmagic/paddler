use crate::agent::state_reporter::StateReporter;
use crate::balancer::status_update::StatusUpdate;
use crate::errors::result::Result;
use crate::llamacpp::llamacpp_client::LlamacppClient;

pub struct Agent {
    id: uuid::Uuid,
    name: Option<String>,
    llamacpp_client: LlamacppClient,
}

impl Agent {
    pub fn new(llamacpp_client: LlamacppClient, name: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            llamacpp_client,
        }
    }

    pub async fn observe_and_report(
        &self,
        state_reporter: &actix::Addr<StateReporter>,
    ) -> Result<()> {
        let status = self.observe().await?;

        Ok(state_reporter.send(status).await?)
    }

    async fn observe(&self) -> Result<StatusUpdate> {
        Ok(StatusUpdate::new(
            self.id,
            self.name.clone(),
            self.llamacpp_client.get_available_slots().await?,
        ))
    }
}
