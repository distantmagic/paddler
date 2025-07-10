use actix_ws::Session;
use anyhow::Result;

use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::Notification;
use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;

pub struct AgentController {
    pub id: String,
    pub name: Option<String>,
    pub session: Session,
}

impl AgentController {
    pub async fn set_desired_state(&mut self, desired_state: LlamaCppDesiredState) -> Result<()> {
        let state_json = serde_json::to_string(&Notification::SetState(SetStateParams {
            desired_state,
        }))?;

        self.session.text(state_json).await?;

        Ok(())
    }
}
