use std::num::NonZeroU32;

use anyhow::Result;
use llama_cpp_bindings::BareJsonToolCalls;
use tokio::sync::mpsc;

use paddler_agent_status::slot_guard::SlotGuard;
use paddler_agent_text_generation::grammar_sampling::GrammarSampling;
use paddler_agent_text_generation::model_bos_token::ModelBosToken;
use paddler_agent_text_generation::prepared_generation_request::PreparedGenerationRequest;
use paddler_agent_text_generation::prepared_prompt::PreparedPrompt;
use paddler_agent_text_generation::prompt_tokenizer::PromptTokenizer;
use paddler_agent_text_generation::sampler_chain_factory::SamplerChainFactory;
use paddler_agent_text_generation::token_classification::TokenClassification;
use paddler_agent_text_generation::token_sampling::TokenSampling;
use paddler_agent_text_generation::tool_call_handling::ToolCallHandling;
use paddler_inference_parameters::sampling_parameters::SamplingParameters;
use paddler_messaging::generated_token_result::GeneratedTokenResult;

use crate::llama_runtime::LlamaRuntime;

pub struct PreparedGeneration {
    pub generate_tokens_stop_tx: mpsc::UnboundedSender<()>,
    pub generated_tokens_rx: mpsc::UnboundedReceiver<GeneratedTokenResult>,
    pub request: PreparedGenerationRequest,
}

impl PreparedGeneration {
    pub fn of_raw_prompt(
        llama_runtime: &LlamaRuntime,
        sequence_context_size: u32,
        raw_prompt: &str,
    ) -> Result<Self> {
        let model = &llama_runtime.loaded_llama_model.model;
        let (generated_tokens_tx, generated_tokens_rx) = mpsc::unbounded_channel();
        let (generate_tokens_stop_tx, generate_tokens_stop_rx) = mpsc::unbounded_channel();

        Ok(Self {
            generate_tokens_stop_tx,
            generated_tokens_rx,
            request: PreparedGenerationRequest {
                generate_tokens_stop_rx,
                generated_tokens_tx,
                max_tokens: NonZeroU32::MIN,
                prompt: PreparedPrompt::TextTokens(
                    PromptTokenizer {
                        model: model.clone(),
                        model_bos_token: ModelBosToken::of(model),
                        sequence_context_size,
                    }
                    .tokenize(raw_prompt)?,
                ),
                slot_guard: SlotGuard::new(
                    llama_runtime
                        .inference_runtime_context
                        .slot_aggregated_status
                        .clone(),
                ),
                token_classification: TokenClassification {
                    bare_json_tool_calls: BareJsonToolCalls::Ignore,
                    streaming_markers: model.streaming_markers()?,
                },
                token_sampling: TokenSampling {
                    chain: SamplerChainFactory {
                        sampling_parameters: SamplingParameters::default(),
                        n_vocab: model.n_vocab(),
                    }
                    .create(0)?,
                    grammar: GrammarSampling::Unconstrained,
                },
                tool_call_handling: ToolCallHandling::Streamed,
            },
        })
    }
}
