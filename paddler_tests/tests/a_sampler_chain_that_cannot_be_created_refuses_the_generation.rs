#![cfg(feature = "tests_that_use_llms")]

use std::num::NonZeroU32;

use tokio::sync::mpsc;

use paddler_agent_runtime::agent_request::AgentRequest;
use paddler_agent_runtime::prepares_scheduler_command::PreparesSchedulerCommand as _;
use paddler_agent_status::slot_guard::SlotGuard;
use paddler_agent_text_generation::json_tools_serializer::JsonToolsSerializer;
use paddler_agent_text_generation::text_generation_request::TextGenerationRequest;
use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_messaging::request_params::continue_from_raw_prompt_params::ContinueFromRawPromptParams;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;
use paddler_tests::sampler_chain_factory_that_fails::SamplerChainFactoryThatFails;

#[test]
fn a_sampler_chain_that_cannot_be_created_refuses_the_generation() {
    let runtime =
        LlamaRuntime::load(&kev_0_8b_target_path("model.gguf")).expect("the kev model must load");
    let preparer = runtime
        .text_only_generation_request_preparer(SamplerChainFactoryThatFails, JsonToolsSerializer)
        .expect("the preparer must be assembled");
    let (generated_tokens_tx, mut generated_tokens_rx) = mpsc::unbounded_channel();
    let (_generate_tokens_stop_tx, generate_tokens_stop_rx) = mpsc::unbounded_channel();

    let scheduler_command = preparer.prepare_scheduler_command(
        Some("test-agent"),
        TextGenerationRequest::ContinueFromRawPrompt(AgentRequest {
            params: ContinueFromRawPromptParams {
                grammar: None,
                max_tokens: NonZeroU32::MIN,
                raw_prompt: "Paddler".to_owned(),
            },
            response_tx: generated_tokens_tx,
            slot_guard: SlotGuard::new(runtime.inference_runtime_context.slot_aggregated_status),
            stop_rx: generate_tokens_stop_rx,
        }),
    );

    assert!(scheduler_command.is_none());
    assert_eq!(
        generated_tokens_rx.try_recv(),
        Ok(GeneratedTokenResult::SamplerChainCreationFailed(
            "test-agent: failed to create sampler chain: chain sampler could not be initialized"
                .to_owned()
        ))
    );
}
