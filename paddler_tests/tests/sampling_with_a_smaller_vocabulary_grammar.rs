#![cfg(feature = "tests_that_use_llms")]

use std::path::Path;

use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::model::AddBos;
use llama_cpp_bindings::model::ParseSpecialTokens;
use llama_cpp_bindings::sampling::LlamaSampler;
use llama_cpp_bindings::token::data_array::LlamaTokenDataArray;

use paddler_agent_text_generation::sampling_outcome::SamplingOutcome;
use paddler_agent_text_generation::text_generation_error::TextGenerationError;
use paddler_agent_text_generation::token_sampling::TokenSampling;

use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;

use crate::vocabulary_only_llama_model::VocabularyOnlyLlamaModel;

pub struct SamplingWithASmallerVocabularyGrammar {
    pub runtime: LlamaRuntime,
    pub smaller_vocabulary_model: VocabularyOnlyLlamaModel,
}

impl SamplingWithASmallerVocabularyGrammar {
    pub fn load(smaller_vocabulary_model_path: &Path) -> Self {
        let runtime = LlamaRuntime::load(
            &kev_0_8b_target_path("model.gguf"),
            agent_inference_runtime_context(1),
        )
        .expect("the kev model must load");
        let smaller_vocabulary_model = VocabularyOnlyLlamaModel::load(
            &runtime.inference_runtime_context.llama_backend,
            smaller_vocabulary_model_path,
        );

        Self {
            runtime,
            smaller_vocabulary_model,
        }
    }

    pub fn sample_the_prompt_with(
        &self,
        token_sampling: impl FnOnce(LlamaSampler) -> TokenSampling,
    ) -> Result<SamplingOutcome, TextGenerationError> {
        let mut llama_context = self
            .runtime
            .create_text_generation_llama_context(1)
            .expect("the text generation context must be created");
        let prompt_tokens = self
            .runtime
            .loaded_llama_model
            .model
            .str_to_token(
                "Paddler balances",
                AddBos::Always,
                ParseSpecialTokens::Always,
            )
            .expect("the prompt must tokenize");
        let mut batch =
            LlamaBatch::new(self.runtime.model_runtime_parameters.n_batch.capacity(), 1);

        batch
            .add_sequence(&prompt_tokens, 0, false)
            .expect("the prompt must fit the batch");
        llama_context
            .decode(&mut batch)
            .expect("the prompt must decode");

        token_sampling(
            LlamaSampler::grammar(&self.smaller_vocabulary_model, r#"root ::= "a""#, "root")
                .expect("the grammar of the smaller vocabulary must initialize"),
        )
        .sample(
            &llama_context,
            batch.n_tokens() - 1,
            &mut LlamaTokenDataArray::new(Vec::new(), false),
        )
    }
}
