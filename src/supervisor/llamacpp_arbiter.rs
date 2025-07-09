use std::net::SocketAddr;
use std::sync::Arc;

use actix::sync::SyncArbiter;
use actix::Addr;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;

use crate::supervisor::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::supervisor::llamacpp_slot::LlamaCppSlot;

pub struct LlamaCppArbiter {
    applicable_state: LlamaCppApplicableState,
}

impl LlamaCppArbiter {
    pub fn new(
        applicable_state: LlamaCppApplicableState,
        llamacpp_listen_addr: SocketAddr,
    ) -> Result<Self> {
        Ok(Self {
            applicable_state,
        })
    }

    pub async fn spawn(&self) -> Result<Addr<LlamaCppSlot>> {
        let backend = Arc::new(LlamaBackend::init()?);
        let params = LlamaModelParams::default();
        let model = Arc::new(LlamaModel::load_from_file(
            &backend.clone(),
            self.applicable_state.model_path.clone(),
            &params,
        )?);
        let ctx_params = Arc::new(LlamaContextParams::default());

        Ok(SyncArbiter::start(3, move || {
            LlamaCppSlot::new(backend.clone(), ctx_params.clone(), model.clone())
        }))
    }
}

#[cfg(test)]
mod tests {
    use futures::future::join_all;

    use super::*;
    use crate::supervisor::converts_to_applicable_state::ConvertsToApplicableState as _;
    use crate::supervisor::huggingface_model_reference::HuggingFaceModelReference;
    use crate::supervisor::llamacpp_desired_model::LlamaCppDesiredModel;
    use crate::supervisor::llamacpp_desired_state::LlamaCppDesiredState;
    use crate::supervisor::message::Generate;

    #[actix_web::test]
    async fn test_llamacpp_arbiter_spawn() -> Result<()> {
        let desired_state = LlamaCppDesiredState {
            model: LlamaCppDesiredModel::HuggingFace(HuggingFaceModelReference {
                filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
                repo: "Qwen/Qwen3-0.6B-GGUF".to_string(),
                // filename: "Qwen3-8B-Q4_K_M.gguf".to_string(),
                // repo: "Qwen/Qwen3-8B-GGUF".to_string(),
            }),
        };

        let applicable_state = desired_state
            .to_applicable_state()
            .await?
            .expect("Failed to convert to applicable state");

        let llamacpp_arbiter =
            LlamaCppArbiter::new(applicable_state, "127.0.0.1:8080".parse::<SocketAddr>()?)?;

        let addr = llamacpp_arbiter.spawn().await?;

        let prompt =
            "<|im_start|>user\nHow can I make a cat happy?<|im_end|>\n<|im_start|>assistant\n";

        let futures = vec![
            addr.send(Generate {
                prompt: prompt.to_string(),
            }),
            addr.send(Generate {
                prompt: prompt.to_string(),
            }),
            addr.send(Generate {
                prompt: prompt.to_string(),
            }),
        ];

        let results = join_all(futures).await;

        for result in results {
            match result {
                Ok(response) => {
                    println!("Response: {}", response?);
                }
                Err(err) => {
                    eprintln!("Error generating response: {err}");
                }
            }
        }

        assert!(false);

        Ok(())
    }
}
