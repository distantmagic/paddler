use reqwest::StatusCode;
use serde_json::json;
use serde_json::to_string;
use tokio::spawn;
use tokio_tungstenite::tungstenite::Message;

use paddler_messaging::agent_runtime_status::AgentRuntimeStatus;
use paddler_messaging::agent_status::AgentStatus;
use paddler_messaging::decision_answer::DecisionAnswer;
use paddler_messaging::decision_result::DecisionResult;
use paddler_messaging::decision_summary::DecisionSummary;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_messaging::jsonrpc::response_envelope::ResponseEnvelope;
use paddler_messaging::management_socket::agent::response::Response as AgentJsonRpcResponse;
use paddler_messaging::management_socket::balancer::message::Message as ManagementJsonRpcMessage;
use paddler_messaging::management_socket::balancer::notification::Notification as ManagementJsonRpcNotification;
use paddler_messaging::management_socket::balancer::notification_params::register_agent_params::RegisterAgentParams;
use paddler_messaging::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;
use paddler_test_cluster_harness::noul_system_one_request::noul_system_one_request;
use paddler_test_cluster_harness::untrusted_agent_socket_client::UntrustedAgentSocketClient;
use paddler_tests::cluster_without_agents_serving::cluster_without_agents_serving;
use paddler_tests::start_cluster::start_cluster;
use paddler_typesafe_translation::typesafe_translation_error::TypeSafeTranslationError;

const DECISION_SLOTS: u16 = 2;

#[tokio::test(flavor = "multi_thread")]
async fn typesafe_system_one_reports_answers_that_do_not_match_its_questions_as_a_bad_gateway() {
    let cluster = start_cluster(cluster_without_agents_serving(
        InferenceMode::TextGenerationEmbeddingsAndDecisions,
    ))
    .await
    .expect("a decision balancer without agents must start");
    let mut untruthful_agent =
        UntrustedAgentSocketClient::connect(cluster.balancer.addresses.management, "untruthful")
            .await
            .expect("the untruthful agent must reach the agent socket");
    let agent_message = |message: ManagementJsonRpcMessage| {
        Message::text(to_string(&message).expect("the agent message must serialize"))
    };

    untruthful_agent
        .send(agent_message(ManagementJsonRpcMessage::Notification(
            ManagementJsonRpcNotification::RegisterAgent(RegisterAgentParams {
                name: Some("untruthful".to_owned()),
                slot_aggregated_status_snapshot: SlotAggregatedStatusSnapshot {
                    status: AgentStatus {
                        desired_slots_total: DECISION_SLOTS,
                        runtime: AgentRuntimeStatus::Serving {
                            inference_mode: InferenceMode::TextGenerationEmbeddingsAndDecisions,
                            slots_total: u64::from(DECISION_SLOTS),
                        },
                        ..AgentStatus::default()
                    },
                    version: 1,
                },
            }),
        )))
        .await
        .expect("the registration must be sent");

    let typesafe_response =
        spawn(cluster.typesafe_system_one("mismatched-answers", &noul_system_one_request()));
    let decision_request_id = untruthful_agent
        .next_request()
        .await
        .expect("the balancer must dispatch the decision")
        .id;

    for decision_result in [
        DecisionResult::QuestionAnswered(DecisionAnswer {
            id: "unasked".to_owned(),
            probabilities: vec![0.5, 0.5],
        }),
        DecisionResult::Done(DecisionSummary {
            input_tokens: 1,
            processing_milliseconds: 1,
        }),
    ] {
        untruthful_agent
            .send(agent_message(ManagementJsonRpcMessage::Response(
                ResponseEnvelope {
                    generated_by: None,
                    request_id: decision_request_id.clone(),
                    response: AgentJsonRpcResponse::Decision(decision_result),
                },
            )))
            .await
            .expect("the decision result must be sent");
    }

    let response = typesafe_response
        .await
        .expect("the TypeSafe request task must not panic")
        .expect("the request must be answered");

    assert_eq!(
        (response.status, response.body),
        (
            StatusCode::BAD_GATEWAY,
            json!({
                "detail": TypeSafeTranslationError::AnswerOutOfOrder {
                    expected_question_id: "paid".to_owned(),
                    received_question_id: "unasked".to_owned(),
                }
                .to_string()
            })
        )
    );

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
