#![cfg(feature = "tests_that_use_llms")]

use llama_cpp_bindings::error::MetaValError;
use tokio_util::sync::CancellationToken;

use paddler_agent_decision::decision_error::DecisionError;
use paddler_agent_decision::decision_pipeline::DecisionPipeline;
use paddler_agent_decision::loaded_pointer_head::LoadedPointerHead;
use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::kev_0_8b_pointer_head::kev_0_8b_pointer_head;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;
use paddler_tests::model_architecture_that_is_unreadable::ModelArchitectureThatIsUnreadable;
use paddler_tests::model_architecture_that_is_unreadable::UNREADABLE_ARCHITECTURE_RETURN_CODE;

#[tokio::test(flavor = "multi_thread")]
async fn a_decision_pipeline_refuses_a_model_whose_architecture_is_unreadable() {
    let LlamaRuntime {
        inference_runtime_context,
        loaded_llama_model,
        model_runtime_parameters,
    } = LlamaRuntime::load(
        &kev_0_8b_target_path("model.gguf"),
        agent_inference_runtime_context(DECISION_SLOTS_MINIMUM.get()),
    )
    .expect("the kev decision model must load");

    assert_eq!(
        DecisionPipeline {
            architecture: ModelArchitectureThatIsUnreadable,
            inference_runtime_context,
            loaded_llama_model,
            loaded_pointer_head: LoadedPointerHead {
                pointer_head: kev_0_8b_pointer_head().expect("the kev pointer head must load"),
                pointer_head_path: kev_0_8b_target_path("pointer_head.gguf"),
            },
            model_runtime_parameters,
        }
        .spawn(&CancellationToken::new())
        .await
        .err()
        .as_ref()
        .map(ToString::to_string),
        Some(
            DecisionError::ModelArchitectureUnreadable(MetaValError::NegativeReturn(
                UNREADABLE_ARCHITECTURE_RETURN_CODE
            ))
            .to_string()
        )
    );
}
