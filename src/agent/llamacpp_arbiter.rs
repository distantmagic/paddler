use std::sync::Arc;

use actix::sync::SyncArbiter;
use anyhow::Result;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::agent::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::agent::llamacpp_arbiter_controller::LlamaCppArbiterController;
use crate::agent::llamacpp_slot::LlamaCppSlot;

pub struct LlamaCppArbiter {
    slots_total: usize,
}

impl LlamaCppArbiter {
    pub fn new(slots_total: usize) -> Self {
        Self {
            slots_total,
        }
    }

    pub async fn spawn(&self) -> Result<LlamaCppArbiterController> {
        let backend = Arc::new(LlamaBackend::init()?);
        let ctx_params = Arc::new(LlamaContextParams::default());
        let (applicable_state_tx, mut applicable_state_rx) =
            mpsc::channel::<LlamaCppApplicableState>(100);
        let (model_tx, _) = broadcast::channel::<Arc<LlamaModel>>(100);

        let backend_clone = backend.clone();
        let model_tx_clone = model_tx.clone();

        tokio::spawn(async move {
            while let Some(applicable_state) = applicable_state_rx.recv().await {
                model_tx_clone
                    .send(Arc::new(
                        LlamaModel::load_from_file(
                            &backend_clone.clone(),
                            applicable_state.model_path.clone(),
                            &LlamaModelParams::default(),
                        )
                        .expect("Failed to load model from file"),
                    ))
                    .expect("Failed to send model");
            }
        });

        Ok(LlamaCppArbiterController {
            applicable_state_tx,
            llamacpp_slot_addr: SyncArbiter::start(self.slots_total, move || {
                LlamaCppSlot::new(backend.clone(), ctx_params.clone(), model_tx.subscribe())
                    .expect("Failed to create LlamaCppSlot")
            }),
        })
    }
}

#[cfg(test)]
#[cfg(feature = "tests_that_use_llms")]
mod tests {
    use futures::future::join_all;
    use tokio::sync::mpsc;

    use super::*;
    use crate::agent::converts_to_applicable_state::ConvertsToApplicableState as _;
    use crate::agent::huggingface_model_reference::HuggingFaceModelReference;
    use crate::agent::llamacpp_desired_model::LlamaCppDesiredModel;
    use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;
    use crate::agent::message::GenerateTokens;

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

        let llamacpp_arbiter = LlamaCppArbiter::new(3);
        let controller = llamacpp_arbiter.spawn().await?;

        let prompt =
            "<|im_start|>user\nHow can I make a cat happy?<|im_end|>\n<|im_start|>assistant\n";
        let (tx, mut rx) = mpsc::channel(100);

        let futures = vec![
            controller.llamacpp_slot_addr.send(GenerateTokens {
                chunk_sender: tx.clone(),
                max_tokens: 100,
                prompt: prompt.to_string(),
            }),
            controller.llamacpp_slot_addr.send(GenerateTokens {
                chunk_sender: tx.clone(),
                max_tokens: 100,
                prompt: prompt.to_string(),
            }),
            controller.llamacpp_slot_addr.send(GenerateTokens {
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
            if let Err(err) = result {
                eprintln!("Error generating response: {err}");
            }
        }

        assert!(false);

        Ok(())
    }
}
