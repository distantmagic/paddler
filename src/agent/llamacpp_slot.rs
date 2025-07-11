use std::sync::Arc;

use actix::Actor;
use actix::Handler;
use actix::SyncContext;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::Special;
use llama_cpp_2::sampling::LlamaSampler;
use log::error;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::task;

use crate::agent::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::agent::message::GenerateTokens;

fn do_generate_tokens(
    ctx: &mut LlamaContext<'_>,
    GenerateTokens {
        chunk_sender,
        max_tokens,
        prompt,
    }: GenerateTokens,
    model: Arc<LlamaModel>,
) -> Result<String> {
    let tokens_list = model.str_to_token(&prompt, AddBos::Always)?;
    let mut batch = LlamaBatch::new(512, 1);
    let last_index = tokens_list.len() as i32 - 1;

    for (i, token) in (0_i32..).zip(tokens_list.into_iter()) {
        let is_last = i == last_index;

        batch.add(token, i, &[0], is_last)?;
    }

    ctx.decode(&mut batch)?;

    let mut n_cur = batch.n_tokens();
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut sampler = LlamaSampler::greedy();
    let mut response: String = String::with_capacity(2048);

    while n_cur <= max_tokens {
        // sample the next token
        {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);

            sampler.accept(token);

            if token == model.token_eos() {
                break;
            }

            let output_bytes = model.token_to_bytes(token, Special::Tokenize)?;
            let mut output_string = String::with_capacity(32);
            let _decode_result = decoder.decode_to_string(&output_bytes, &mut output_string, false);

            response.push_str(&output_string);
            chunk_sender.blocking_send(output_string)?;

            batch.clear();
            batch.add(token, n_cur, &[0], true)?;
        }

        n_cur += 1;

        ctx.decode(&mut batch)?;
    }

    Ok(response)
}

pub struct LlamaCppSlot {
    backend: Arc<LlamaBackend>,
    ctx_params: Arc<LlamaContextParams>,
    generate_tokens_tx: mpsc::Sender<GenerateTokens>,
}

impl LlamaCppSlot {
    pub fn new(
        backend: Arc<LlamaBackend>,
        ctx_params: Arc<LlamaContextParams>,
        mut model_rx: broadcast::Receiver<Arc<LlamaModel>>,
    ) -> Result<Self> {
        let backend_clone = backend.clone();
        let ctx_params_clone = ctx_params.clone();
        let (generate_tokens_tx, mut generate_tokens_rx) = mpsc::channel::<GenerateTokens>(100);

        tokio::task::spawn_local(async move {
            let mut current_ctx: Option<LlamaContext<'_>> = None;
            let mut current_model: Option<Arc<LlamaModel>> = None;

            loop {
                tokio::select! {
                    generate_tokens = generate_tokens_rx.recv() => {
                        do_generate_tokens(
                            current_ctx.as_mut().expect("Context must be initialized"),
                            generate_tokens.expect("Failed to receive GenerateTokens"),
                            current_model.as_ref().expect("Model must be initialized").clone(),
                        ).expect("Failed to generate tokens");
                    }
                    model = model_rx.recv() => match model {
                        Ok(model) => {
                            debug_assert!(
                                Arc::strong_count(&model) >= 1,
                                "Model Arc must have at least one reference"
                            );

                            current_ctx = unsafe {
                                // SAFETY: Extending the lifetime of the model reference to 'static.
                                // This should be safe because:
                                // 1. The model is stored in an Arc, so it won't be deallocated
                                // 2. We store the Arc in the same struct, ensuring it lives as long as the context
                                // 3. The context cannot outlive the struct that contains both it and the model
                                let model_ref: &'static LlamaModel = std::mem::transmute(model.as_ref());

                                Some(model_ref.new_context(&backend_clone, (*ctx_params_clone).clone()).expect("Failed to create context"))
                            };
                            current_model = Some(model.clone());
                        }
                        Err(err) => {
                            error!("Failed to receive model: {err}");
                        }
                    }
                }
            }
        });

        Ok(Self {
            backend,
            ctx_params,
            generate_tokens_tx,
        })
    }
}

impl Actor for LlamaCppSlot {
    type Context = SyncContext<Self>;
}

impl Handler<GenerateTokens> for LlamaCppSlot {
    type Result = Result<()>;

    fn handle(
        &mut self,
        generate_tokens: GenerateTokens,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Ok(self.generate_tokens_tx.blocking_send(generate_tokens)?)
    }
}
