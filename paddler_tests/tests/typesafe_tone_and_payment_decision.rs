use anyhow::Result;
use serde_json::json;

use paddler_test_cluster_harness::typesafe_api_response::TypeSafeApiResponse;

use crate::decision_cluster_params::DecisionClusterParams;
use crate::start_decision_cluster::start_decision_cluster;

pub const TYPESAFE_TONE_AND_PAYMENT_REQUEST_ID: &str = "typesafe-request";

pub async fn typesafe_tone_and_payment_decision() -> Result<TypeSafeApiResponse> {
    let cluster = start_decision_cluster(DecisionClusterParams::default()).await?;
    let typesafe_decision = cluster
        .typesafe_system_one(
            TYPESAFE_TONE_AND_PAYMENT_REQUEST_ID,
            &json!({
                "model": "kev-latest",
                "state": "The invoice was paid on time.",
                "questions": {
                    "tone": {"type": "choice", "criteria": {"calm": null, "angry": "Shouting"}},
                    "paid": {"type": "noul", "instructions": "Was it paid?"},
                },
            }),
        )
        .await?;

    cluster.shutdown().await?;

    Ok(typesafe_decision)
}
