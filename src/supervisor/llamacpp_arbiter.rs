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
    slots_total: usize,
}

impl LlamaCppArbiter {
    pub fn new(applicable_state: LlamaCppApplicableState, slots_total: usize) -> Result<Self> {
        Ok(Self {
            applicable_state,
            slots_total,
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

        Ok(SyncArbiter::start(self.slots_total, move || {
            LlamaCppSlot::new(backend.clone(), ctx_params.clone(), model.clone())
                .expect("Failed to create LlamaCppSlot")
        }))
    }
}

#[cfg(test)]
#[cfg(feature = "tests_that_use_llms")]
mod tests {
    use futures::future::join_all;
    use tokio::sync::mpsc;

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

        let llamacpp_arbiter = LlamaCppArbiter::new(applicable_state, 3)?;

        let addr = llamacpp_arbiter.spawn().await?;

        let prompt =
            "<|im_start|>user\nHow can I make a cat happy?<|im_end|>\n<|im_start|>assistant\n";
        let (tx, mut rx) = mpsc::channel(100);

        let futures = vec![
            addr.send(Generate {
                chunk_sender: tx.clone(),
                max_tokens: 100,
                prompt: prompt.to_string(),
            }),
            addr.send(Generate {
                chunk_sender: tx.clone(),
                max_tokens: 100,
                prompt: prompt.to_string(),
            }),
            addr.send(Generate {
                chunk_sender: tx,
                max_tokens: 100,
                prompt: prompt.to_string(),
            }),
        ];

        tokio::spawn(async move {
            while let Some(chunk) = rx.recv().await {
                println!("Received chunk: {chunk}");
            }
        });

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
