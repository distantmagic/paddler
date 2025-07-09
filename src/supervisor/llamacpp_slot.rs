use std::io::stdout;
use std::io::Write as _;
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
use uuid::Uuid;

use crate::supervisor::message::Generate;

pub struct LlamaCppSlot {
    ctx: LlamaContext<'static>,
    id: Uuid,
    model: Arc<LlamaModel>,
}

impl LlamaCppSlot {
    pub fn new(
        backend: Arc<LlamaBackend>,
        ctx_params: Arc<LlamaContextParams>,
        model: Arc<LlamaModel>,
    ) -> Self {
        debug_assert!(
            Arc::strong_count(&model) >= 1,
            "Model Arc must have at least one reference"
        );

        let ctx = unsafe {
            // SAFETY: Extending the lifetime of the model reference to 'static.
            // This should be safe because:
            // 1. The model is stored in an Arc, so it won't be deallocated
            // 2. We store the Arc in the same struct, ensuring it lives as long as the context
            // 3. The context cannot outlive the struct that contains both it and the model
            let model_ref: &'static LlamaModel = std::mem::transmute(model.as_ref());

            model_ref
                .new_context(&backend, (*ctx_params).clone())
                .unwrap()
        };

        Self {
            ctx,
            id: Uuid::new_v4(),
            model,
        }
    }
}

impl Actor for LlamaCppSlot {
    type Context = SyncContext<Self>;
}

impl Handler<Generate> for LlamaCppSlot {
    type Result = Result<String>;

    fn handle(
        &mut self,
        Generate {
            prompt,
        }: Generate,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let id = self.id.to_string();

        let tokens_list = self.model.str_to_token(&prompt, AddBos::Always)?;
        let n_len = 500;
        let mut batch = LlamaBatch::new(512, 1);
        let last_index = tokens_list.len() as i32 - 1;

        for (i, token) in (0_i32..).zip(tokens_list.into_iter()) {
            let is_last = i == last_index;
            batch.add(token, i, &[0], is_last)?;
        }

        self.ctx.decode(&mut batch)?;

        let mut n_cur = batch.n_tokens();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut sampler = LlamaSampler::greedy();
        let mut response: String = String::with_capacity(2048);

        while n_cur <= n_len {
            // sample the next token
            {
                let token = sampler.sample(&self.ctx, batch.n_tokens() - 1);

                sampler.accept(token);

                // is it an end of stream?
                if token == self.model.token_eos() {
                    break;
                }

                let output_bytes = self.model.token_to_bytes(token, Special::Tokenize).unwrap();
                // use `Decoder.decode_to_string()` to avoid the intermediate buffer
                let mut output_string = String::with_capacity(32);
                let _decode_result =
                    decoder.decode_to_string(&output_bytes, &mut output_string, false);

                println!("slot {id}: {output_string}");

                response.push_str(&output_string);

                stdout().flush().unwrap();

                batch.clear();
                batch.add(token, n_cur, &[0], true).unwrap();
            }

            n_cur += 1;

            self.ctx.decode(&mut batch).expect("failed to eval");
        }

        Ok(response)
    }
}
