#![cfg(feature = "tests_that_use_llms")]

use paddler_agent_text_generation::chat_template_renderer::ChatTemplateRenderer;
use paddler_tests::first_message_content_chat_template::first_message_content_chat_template;
use paddler_tests::llama_runtime::LlamaRuntime;
use paddler_tests::text_only_generation_parts::TextOnlyGenerationParts;

pub fn first_message_content_generation_parts(
    llama_runtime: &LlamaRuntime,
    sequence_context_size: u32,
) -> TextOnlyGenerationParts {
    TextOnlyGenerationParts {
        chat_template_renderer: ChatTemplateRenderer::new(first_message_content_chat_template())
            .expect("the first message content chat template must compile"),
        sequence_context_size,
        streaming_markers: llama_runtime
            .loaded_llama_model
            .model
            .streaming_markers()
            .expect("the model must report its streaming markers"),
    }
}
