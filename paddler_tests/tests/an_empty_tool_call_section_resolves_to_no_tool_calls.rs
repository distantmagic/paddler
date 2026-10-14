#![cfg(feature = "tests_that_use_llms")]

use std::num::NonZeroU32;

use tokio_util::sync::CancellationToken;

use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_test_cluster_harness::agent_config::AgentConfig;
use paddler_tests::gbnf_literal::gbnf_literal;
use paddler_tests::get_weather_tool::get_weather_tool;
use paddler_tests::start_cluster_with_qwen3::start_cluster_with_qwen3;
use paddler_tests::weather_question_conversation::weather_question_conversation;

const FORCED_OUTPUT: &str = "<tool_call></tool_call>";

#[tokio::test(flavor = "multi_thread")]
async fn an_empty_tool_call_section_resolves_to_no_tool_calls() {
    let cluster = start_cluster_with_qwen3(AgentConfig::uniform(1, 1))
        .await
        .expect("the cluster must start");

    let token_results = cluster
        .continue_from_conversation_history(
            CancellationToken::new(),
            &weather_question_conversation(
                gbnf_literal(FORCED_OUTPUT),
                NonZeroU32::try_from(
                    u32::try_from(FORCED_OUTPUT.len())
                        .expect("the forced text length must fit the token limit"),
                )
                .expect("the forced text must not be empty"),
                vec![get_weather_tool()],
            ),
        )
        .await
        .expect("the inference request must be accepted")
        .into_token_results();

    assert_eq!(
        token_results
            .iter()
            .filter(|token_result| token_result.is_tool_call_parsed())
            .collect::<Vec<_>>(),
        vec![&GeneratedTokenResult::ToolCallParsed(Vec::new())]
    );

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
