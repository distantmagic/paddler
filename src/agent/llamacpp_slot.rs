use std::path::PathBuf;
use std::sync::Arc;

use actix::Actor;
use actix::Handler;
use actix::SyncContext;
use anyhow::anyhow;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaChatMessage;
use llama_cpp_2::model::LlamaChatTemplate;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::Special;
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::DecodeError;
use log::debug;
use log::info;
use tokio::sync::mpsc;

use crate::agent::continue_conversation_request::ContinueConversationRequest;
use crate::agent::dispenses_slots::DispensesSlots as _;
use crate::agent::generate_tokens_drop_guard::GenerateTokensDropGuard;
use crate::agent::generate_tokens_request::GenerateTokensRequest;
use crate::agent::kv_cache_repair_action::KVCacheRepairAction;
use crate::agent::slot_status::SlotStatus;
use crate::generated_token::GeneratedToken;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::generated_token_result::GeneratedTokenResult;
use crate::model_parameters::ModelParameters;
use crate::request_params::GenerateTokensParams;

pub struct LlamaCppSlot {
    agent_name: Option<String>,
    llama_chat_template: Arc<LlamaChatTemplate>,
    llama_context: LlamaContext<'static>,
    model: Arc<LlamaModel>,
    model_parameters: ModelParameters,
    model_path: PathBuf,
    slot_index: u32,
    slot_status: Arc<SlotStatus>,
}

impl LlamaCppSlot {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        agent_name: Option<String>,
        backend: Arc<LlamaBackend>,
        ctx_params: Arc<LlamaContextParams>,
        llama_chat_template: Arc<LlamaChatTemplate>,
        model: Arc<LlamaModel>,
        model_parameters: ModelParameters,
        model_path: PathBuf,
        slot_index: u32,
        slot_status: Arc<SlotStatus>,
    ) -> Result<Self> {
        debug_assert!(
            Arc::strong_count(&model) >= 1,
            "Model Arc must have at least one reference"
        );

        let llama_context = unsafe {
            // SAFETY: Extending the lifetime of the model reference to 'static.
            // This should be safe because:
            // 1. The model is stored in an Arc, so it won't be deallocated
            // 2. We store the Arc in the same struct, ensuring it lives as long as the context
            // 3. The context cannot outlive the struct that contains both it and the model
            let model_ref: &'static LlamaModel = std::mem::transmute(model.as_ref());

            model_ref.new_context(&backend, (*ctx_params).clone())?
        };

        Ok(Self {
            agent_name,
            llama_chat_template,
            llama_context,
            model,
            model_parameters,
            model_path,
            slot_index,
            slot_status,
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
                            self.agent_name, self.slot_index
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
                        self.agent_name, self.slot_index,
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

    fn generate_from_prompt(
        &mut self,
        mut generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
        generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
        max_tokens: i32,
        prompt: String,
    ) -> Result<()> {
        self.slot_status.take_slot();

        let _guard = GenerateTokensDropGuard::new(
            generated_tokens_tx.clone(),
            self.slot_index,
            self.slot_status.clone(),
        );

        self.llama_context.clear_kv_cache();

        let tokens_list = self.model.str_to_token(&prompt, AddBos::Always)?;
        let mut batch = LlamaBatch::new(self.model_parameters.batch_n_tokens, 1);
        let last_index = tokens_list.len() as i32 - 1;

        for (i, token) in (0_i32..).zip(tokens_list.into_iter()) {
            let is_last = i == last_index;

            batch.add(token, i, &[0], is_last)?;
        }

        self.decode_batch(&mut batch, &mut vec![])?;

        let mut n_cur = batch.n_tokens();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(self.model_parameters.temperature),
            LlamaSampler::top_k(self.model_parameters.top_k),
            LlamaSampler::top_p(self.model_parameters.top_p, 0),
            LlamaSampler::min_p(self.model_parameters.min_p, 0),
            LlamaSampler::penalties(
                self.model_parameters.penalty_last_n,
                self.model_parameters.penalty_repeat,
                self.model_parameters.penalty_frequency,
                self.model_parameters.penalty_presence,
            ),
            LlamaSampler::greedy(),
        ]);

        while n_cur <= max_tokens {
            if generate_tokens_stop_rx.try_recv().is_ok() {
                debug!(
                    "{:?}: slot {} received stop signal",
                    self.agent_name, self.slot_index
                );

                break;
            }

            // sample the next token
            {
                let token = sampler.sample(&self.llama_context, batch.n_tokens() - 1);

                sampler.accept(token);

                if token == self.model.token_eos() {
                    break;
                }

                let output_bytes = self.model.token_to_bytes(token, Special::Tokenize)?;
                let mut output_string = String::with_capacity(32);
                let _decode_result =
                    decoder.decode_to_string(&output_bytes, &mut output_string, false);

                generated_tokens_tx.send(GeneratedTokenEnvelope {
                    slot: self.slot_index,
                    generated_token_result: GeneratedTokenResult::Token(GeneratedToken {
                        token: output_string,
                    }),
                })?;

                batch.clear();
                batch.add(token, n_cur, &[0], true)?;
            }

            n_cur += 1;

            self.decode_batch(&mut batch, &mut vec![])?;
        }

        Ok(())
    }
}

impl Actor for LlamaCppSlot {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.slot_status.started();

        info!(
            "{:?}: slot {} ready with model {:?}",
            self.agent_name,
            self.slot_index,
            self.model_path.display(),
        );
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.slot_status.stopped();

        info!("{:?}: slot {} stopped", self.agent_name, self.slot_index,);
    }
}

impl Handler<ContinueConversationRequest> for LlamaCppSlot {
    type Result = Result<()>;

    fn handle(
        &mut self,
        ContinueConversationRequest {
            continue_conversation_params,
            generate_tokens_stop_rx,
            generated_tokens_tx,
        }: ContinueConversationRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let mut llama_chat_messages: Vec<LlamaChatMessage> =
            Vec::with_capacity(continue_conversation_params.conversation_history.len());

        for message in continue_conversation_params.conversation_history {
            let llama_chat_message = LlamaChatMessage::new(message.role, message.content)?;

            llama_chat_messages.push(llama_chat_message);
        }

        let prompt = self.model.apply_chat_template(
            &self.llama_chat_template,
            &llama_chat_messages,
            true,
        )?;

        self.generate_from_prompt(
            generate_tokens_stop_rx,
            generated_tokens_tx,
            continue_conversation_params.max_tokens,
            prompt,
        )
    }
}

impl Handler<GenerateTokensRequest> for LlamaCppSlot {
    type Result = Result<()>;

    fn handle(
        &mut self,
        GenerateTokensRequest {
            generate_tokens_params: GenerateTokensParams { prompt, max_tokens },
            generate_tokens_stop_rx,
            generated_tokens_tx,
        }: GenerateTokensRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.generate_from_prompt(
            generate_tokens_stop_rx,
            generated_tokens_tx,
            max_tokens,
            prompt,
        )
    }
}
