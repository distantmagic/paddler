use actix_ws::Session;
use anyhow::Result;

use crate::supervisor::jsonrpc::notification_params::SetStateParams;
use crate::supervisor::jsonrpc::Notification;
use crate::supervisor::llamacpp_desired_state::LlamaCppDesiredState;

pub struct SupervisorController {
    pub id: String,
    pub name: Option<String>,
    pub session: Session,
}

impl SupervisorController {
    pub async fn set_desired_state(&mut self, desired_state: LlamaCppDesiredState) -> Result<()> {
        let state_json = serde_json::to_string(&Notification::SetState(SetStateParams {
            desired_state,
        }))?;

        self.session.text(state_json).await?;

        Ok(())
    }
}
