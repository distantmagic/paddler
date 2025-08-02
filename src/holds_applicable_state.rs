use tokio::sync::watch::Receiver;
use anyhow::Result;

pub trait HoldsApplicableState {
    type ApplicableState;

    fn set_applicable_state(
        &self,
        applicable_state: Option<Self::ApplicableState>,
    ) -> Result<()>;

    fn subscribe(&self) -> Receiver<Option<Self::ApplicableState>>;
}
