use reqwest::StatusCode;
use serde_json::json;

use paddler_balancer::compatibility::typesafe_service::typesafe_header::TypeSafeHeader;
use paddler_messaging::agent_desired_model::AgentDesiredModel;

use crate::typesafe_models_listed_for::typesafe_models_listed_for;

#[tokio::test(flavor = "multi_thread")]
async fn typesafe_models_lists_nothing_while_no_model_is_served() {
    let response = typesafe_models_listed_for(AgentDesiredModel::None).await;

    assert_eq!(response.status, StatusCode::OK);
    assert!(
        response
            .headers
            .get(TypeSafeHeader::REQUEST_ID)
            .is_some_and(|request_id| !request_id.is_empty())
    );
    assert_eq!(response.body, json!({"models": []}));
}
