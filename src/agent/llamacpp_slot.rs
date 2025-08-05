use std::sync::Arc;

use actix::Actor;
use actix::Handler;
use actix::SyncContext;
use anyhow::anyhow;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use rand::rngs::ThreadRng;
use llama_cpp_2::llama_batch::LlamaBatch;
use rand::Rng as _;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::Special;
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::DecodeError;
use log::debug;
use log::error;
use log::info;
use minijinja::context;
use tokio::sync::mpsc;

use crate::agent::continue_from_conversation_history_request::ContinueFromConversationHistoryRequest;
use crate::agent::generate_tokens_drop_guard::GenerateTokensDropGuard;
use crate::agent::continue_from_raw_prompt_request::ContinueFromRawPromptRequest;
use crate::agent::kv_cache_repair_action::KVCacheRepairAction;
use crate::dispenses_slots::DispensesSlots as _;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::generated_token_result::GeneratedTokenResult;
use crate::request_params::ContinueFromConversationHistoryParams;
use crate::request_params::ContinueFromRawPromptParams;
use crate::agent::llamacpp_slot_context::LlamaCppSlotContext;
use crate::slot_status::SlotStatus;

pub struct LlamaCppSlot {
    index: u32,
    llama_context: LlamaContext<'static>,
    rng: ThreadRng,
    slot_context: Arc<LlamaCppSlotContext>,
    status: Arc<SlotStatus>,
}

impl LlamaCppSlot {
    pub fn new(
        backend: Arc<LlamaBackend>,
        ctx_params: Arc<LlamaContextParams>,
        index: u32,
        slot_context: Arc<LlamaCppSlotContext>,
        status: Arc<SlotStatus>,
    ) -> Result<Self> {
        debug_assert!(
            Arc::strong_count(&slot_context.model) >= 1,
            "Model Arc must have at least one reference"
        );

        let llama_context = unsafe {
            // SAFETY: Extending the lifetime of the model reference to 'static.
            // This should be safe because:
            // 1. The model is stored in an Arc, so it won't be deallocated
            // 2. We store the Arc in the same struct, ensuring it lives as long as the context
            // 3. The context cannot outlive the struct that contains both it and the model
            let model_ref: &'static LlamaModel = std::mem::transmute(slot_context.model.as_ref());

            model_ref.new_context(&backend, (*ctx_params).clone())?
        };

        Ok(Self {
            index,
            llama_context,
            rng: rand::rng(),
            slot_context,
            status,
        })
    }

    fn decode_batch(
        &mut self,
        batch: &mut LlamaBatch,
        taken_kv_cache_repair_actions: &mut Vec<KVCacheRepairAction>,
    ) -> Result<()> {
        if let Err(err) = self.llama_context.decode(batch) {
            match err {
                DecodeError::NoKvCacheSlot => {
                    if !taken_kv_cache_repair_actions.contains(&KVCacheRepairAction::Defrag) {
                        debug!(
                            "{:?}: slot {} has no KV cache slot, defragmenting",
                            self.slot_context.agent_name, self.index
                        );

                        taken_kv_cache_repair_actions.push(KVCacheRepairAction::Defrag);
                        self.llama_context.kv_cache_defrag();

                        return self.decode_batch(batch, taken_kv_cache_repair_actions);
                    }

                    Err(err.into())
                }
                DecodeError::NTokensZero => {
                    debug!(
                        "{:?}: slot {} - the number of tokens in the batch was 0",
                        self.slot_context.agent_name, self.index,
                    );

                    Err(err.into())
                }
                DecodeError::Unknown(error_code) => {
                    Err(anyhow!("Unknown error code: {error_code}"))
                }
            }
        } else {
            Ok(())
        }
    }

    fn generate_from_raw_prompt(
        &mut self,
        mut generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
        generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
        max_tokens: i32,
        prompt: String,
    ) -> Result<()> {
        self.status.take_slot();

        let _guard = GenerateTokensDropGuard::new(self.status.clone());

        self.llama_context.clear_kv_cache();

        let tokens_list = self.slot_context.model.str_to_token(&prompt, AddBos::Always)?;
        let mut batch = LlamaBatch::new(self.slot_context.inference_parameters.batch_n_tokens, 1);
        let last_index = tokens_list.len() as i32 - 1;

        for (i, token) in (0_i32..).zip(tokens_list.into_iter()) {
            let is_last = i == last_index;

            batch.add(token, i, &[0], is_last)?;
        }

        self.decode_batch(&mut batch, &mut vec![])?;

        let mut n_cur = batch.n_tokens();
        let mut decoder = encoding_rs::UTF_8.new_decoder();

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(
                self.slot_context.inference_parameters.penalty_last_n,
                self.slot_context.inference_parameters.penalty_repeat,
                self.slot_context.inference_parameters.penalty_frequency,
                self.slot_context.inference_parameters.penalty_presence,
            ),
            LlamaSampler::top_k(self.slot_context.inference_parameters.top_k),
            LlamaSampler::top_p(self.slot_context.inference_parameters.top_p, 0),
            LlamaSampler::min_p(self.slot_context.inference_parameters.min_p, 0),
            LlamaSampler::temp(self.slot_context.inference_parameters.temperature),
            LlamaSampler::dist(self.rng.random::<u32>()),
        ]);

        while n_cur <= max_tokens {
            if generate_tokens_stop_rx.try_recv().is_ok() {
                debug!(
                    "{:?}: slot {} received stop signal",
                    self.slot_context.agent_name, self.index
                );

                break;
            }

            // sample the next token
            {
                let token = sampler.sample(&self.llama_context, batch.n_tokens() - 1);

                sampler.accept(token);

                if token == self.slot_context.model.token_eos() {
                    break;
                }

                let output_bytes = self.slot_context.model.token_to_bytes(token, Special::Tokenize)?;
                let mut output_string = String::with_capacity(32);
                let _decode_result =
                    decoder.decode_to_string(&output_bytes, &mut output_string, false);

                generated_tokens_tx.send(GeneratedTokenEnvelope {
                    slot: self.index,
                    generated_token_result: GeneratedTokenResult::Token(output_string),
                })?;

                batch.clear();
                batch.add(token, n_cur, &[0], true)?;
            }

            n_cur += 1;

            self.decode_batch(&mut batch, &mut vec![])?;
        }

        generated_tokens_tx.send(GeneratedTokenEnvelope {
            slot: self.index,
            generated_token_result: GeneratedTokenResult::Done,
        })?;

        Ok(())
    }
}

impl Actor for LlamaCppSlot {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.status.started();

        info!(
            "{:?}: slot {} ready with model {:?}",
            self.slot_context.agent_name,
            self.index,
            self.slot_context.model_path.display(),
        );
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.status.stopped();

        info!("{:?}: slot {} stopped", self.slot_context.agent_name, self.index,);
    }
}

impl Handler<ContinueFromConversationHistoryRequest> for LlamaCppSlot {
    type Result = Result<()>;

    fn handle(
        &mut self,
        ContinueFromConversationHistoryRequest {
            continue_from_conversation_history_params:
                ContinueFromConversationHistoryParams {
                    add_generation_prompt,
                    enable_thinking,
                    conversation_history,
                    max_tokens,
                },
            generate_tokens_stop_rx,
            generated_tokens_tx,
        }: ContinueFromConversationHistoryRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let raw_prompt = match self.slot_context.chat_template_renderer.render(context! {
            // Known uses:
            // https://huggingface.co/unsloth/DeepSeek-R1-0528-Qwen3-8B-GGUF
            add_generation_prompt,
            // Known uses:
            // https://huggingface.co/bartowski/Mistral-7B-Instruct-v0.3-GGUF
            // https://huggingface.co/unsloth/DeepSeek-R1-0528-Qwen3-8B-GGUF
            bos_token => self.slot_context.token_bos_str,
            // Known uses:
            // https://huggingface.co/Qwen/Qwen3-0.6B-GGUF
            enable_thinking,
            // Known uses:
            // https://huggingface.co/bartowski/Mistral-7B-Instruct-v0.3-GGUF
            eos_token => self.slot_context.token_eos_str,
            messages => conversation_history,
            nl_token => self.slot_context.token_nl_str,
        }) {
            Ok(raw_prompt) => raw_prompt,
            Err(err) => {
                let msg = format!(
                    "{:?}: slot {} failed to render chat template: {err:?}",
                    self.slot_context.agent_name, self.index
                );

                error!("{msg}");

                generated_tokens_tx.send(GeneratedTokenEnvelope {
                    slot: self.index,
                    generated_token_result: GeneratedTokenResult::ChatTemplateError(msg),
                })?;

                return Err(err);
            }
        };

        debug!(
            "{:?}: slot {} generating from raw prompt: {:?}",
            self.slot_context.agent_name, self.index, raw_prompt
        );

        self.generate_from_raw_prompt(
            generate_tokens_stop_rx,
            generated_tokens_tx,
            max_tokens,
            raw_prompt,
        )
    }
}

impl Handler<ContinueFromRawPromptRequest> for LlamaCppSlot {
    type Result = Result<()>;

    fn handle(
        &mut self,
        ContinueFromRawPromptRequest {
            continue_from_raw_prompt_params: ContinueFromRawPromptParams { max_tokens, raw_prompt },
            generate_tokens_stop_rx,
            generated_tokens_tx,
        }: ContinueFromRawPromptRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.generate_from_raw_prompt(
            generate_tokens_stop_rx,
            generated_tokens_tx,
            max_tokens,
            raw_prompt,
        )
    }
}
