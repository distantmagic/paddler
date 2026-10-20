use reqwest::StatusCode;
use serde_json::json;

use paddler_balancer::compatibility::typesafe_service::typesafe_header::TypeSafeHeader;
use paddler_messaging::agent_desired_model::AgentDesiredModel;

use crate::typesafe_models_listed_for::typesafe_models_listed_for;

const KEV_MODEL_URI: &str = "https://huggingface.co/intentee/kev/blob/main/kev-0.8b.gguf";

#[tokio::test(flavor = "multi_thread")]
async fn typesafe_models_lists_the_model_the_cluster_serves() {
    let response =
        typesafe_models_listed_for(AgentDesiredModel::Uri(KEV_MODEL_URI.to_owned())).await;

    assert_eq!(response.status, StatusCode::OK);
    assert!(
        response
            .headers
            .get(TypeSafeHeader::REQUEST_ID)
            .is_some_and(|request_id| !request_id.is_empty())
    );
    assert_eq!(
        response.body,
        json!({
            "models": [{
                "name": KEV_MODEL_URI,
                "description": format!("Decision model {KEV_MODEL_URI} served by Paddler"),
                "release_date": "unknown",
            }],
        })
    );
}
