use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

use crate::balancer::inference_client::Message as OutgoingMessage;

#[async_trait]
pub trait TransformsOutgoingMessage {
    type TransformedMessage: Serialize;

    async fn transform(&self, message: OutgoingMessage) -> Result<Self::TransformedMessage>;
}
