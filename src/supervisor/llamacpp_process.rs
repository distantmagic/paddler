use std::io::Write as _;
use std::net::SocketAddr;

use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::Special;
use llama_cpp_2::sampling::LlamaSampler;
use tokio::process::Child;
use tokio::sync::Mutex;

use crate::supervisor::llamacpp_applicable_state::LlamaCppApplicableState;

pub struct LlamaCppProcess {
    applicable_state: LlamaCppApplicableState,
    child_process: Mutex<Option<Child>>,
    llamacpp_listen_addr: SocketAddr,
}

impl LlamaCppProcess {
    pub fn new(
        applicable_state: LlamaCppApplicableState,
        llamacpp_listen_addr: SocketAddr,
    ) -> Result<Self> {
        Ok(Self {
            applicable_state,
            child_process: Mutex::new(None),
            llamacpp_listen_addr,
        })
    }

    pub fn spawn(&self) -> Result<()> {
        let backend = LlamaBackend::init()?;
        let params = LlamaModelParams::default();
        let prompt =
            "<|im_start|>user\nHello! how are you?<|im_end|>\n<|im_start|>assistant\n".to_string();
        let model = LlamaModel::load_from_file(
            &backend,
            self.applicable_state.model_path.clone(),
            &params,
        )?;
        let ctx_params = LlamaContextParams::default();
        let mut ctx = model.new_context(&backend, ctx_params)?;
        let tokens_list = model.str_to_token(&prompt, AddBos::Always)?;
        let n_len = 500;
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

        while n_cur <= n_len {
            // sample the next token
            {
                let token = sampler.sample(&ctx, batch.n_tokens() - 1);

                sampler.accept(token);

                // is it an end of stream?
                if token == model.token_eos() {
                    eprintln!();
                    break;
                }

                let output_bytes = model.token_to_bytes(token, Special::Tokenize).unwrap();
                // use `Decoder.decode_to_string()` to avoid the intermediate buffer
                let mut output_string = String::with_capacity(32);
                let _decode_result =
                    decoder.decode_to_string(&output_bytes, &mut output_string, false);

                print!("{output_string}");

                std::io::stdout().flush().unwrap();

                batch.clear();
                batch.add(token, n_cur, &[0], true).unwrap();
            }

            n_cur += 1;

            ctx.decode(&mut batch).expect("failed to eval");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::supervisor::converts_to_applicable_state::ConvertsToApplicableState as _;
    use crate::supervisor::huggingface_model_reference::HuggingFaceModelReference;
    use crate::supervisor::llamacpp_desired_model::LlamaCppDesiredModel;
    use crate::supervisor::llamacpp_desired_state::LlamaCppDesiredState;

    #[tokio::test]
    async fn test_llamacpp_process_spawn() -> Result<()> {
        let desired_state = LlamaCppDesiredState {
            model: LlamaCppDesiredModel::HuggingFace(HuggingFaceModelReference {
                filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
                repo: "Qwen/Qwen3-0.6B-GGUF".to_string(),
            }),
        };

        let applicable_state = desired_state
            .to_applicable_state()
            .await?
            .expect("Failed to convert to applicable state");

        let llamacpp_process =
            LlamaCppProcess::new(applicable_state, "127.0.0.1:8080".parse::<SocketAddr>()?)?;

        llamacpp_process.spawn()?;

        assert!(false);

        Ok(())
    }
}
