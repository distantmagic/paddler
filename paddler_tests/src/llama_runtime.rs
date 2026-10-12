use std::path::Path;

use anyhow::Result;
use llama_cpp_bindings::context::LlamaContext;
use llama_cpp_bindings::model::params::LlamaModelParams;

use paddler_agent_decision::decision_llama_context_settings::DecisionLlamaContextSettings;
use paddler_agent_runtime::inference_runtime_context::InferenceRuntimeContext;
use paddler_agent_runtime::inference_thread_count::inference_thread_count;
use paddler_agent_runtime::llama_context_settings::LlamaContextSettings;
use paddler_agent_runtime::loaded_llama_model::LoadedLlamaModel;
use paddler_inference_parameters::all_gpu_layers::ALL_GPU_LAYERS;
use paddler_inference_parameters::model_runtime_parameters::ModelRuntimeParameters;

use crate::agent_inference_runtime_context::agent_inference_runtime_context;

pub struct LlamaRuntime {
    pub inference_runtime_context: InferenceRuntimeContext,
    pub loaded_llama_model: LoadedLlamaModel,
    pub model_runtime_parameters: ModelRuntimeParameters,
}

impl LlamaRuntime {
    pub fn load(model_path: &Path) -> Result<Self> {
        let inference_runtime_context = agent_inference_runtime_context(1);
        let model_runtime_parameters = ModelRuntimeParameters {
            n_gpu_layers: ALL_GPU_LAYERS,
            ..ModelRuntimeParameters::default()
        };
        let model_params =
            LlamaModelParams::default().with_n_gpu_layers(model_runtime_parameters.n_gpu_layers);
        let loaded_llama_model =
            LoadedLlamaModel::load(&inference_runtime_context, model_path, &model_params)?;

        Ok(Self {
            inference_runtime_context,
            loaded_llama_model,
            model_runtime_parameters,
        })
    }

    pub fn create_decision_llama_context(&self, n_seq_max: u32) -> Result<LlamaContext<'_>> {
        Ok(DecisionLlamaContextSettings {
            llama_context_settings: self.llama_context_settings(n_seq_max)?,
        }
        .create_llama_context(&self.loaded_llama_model, &self.inference_runtime_context)?)
    }

    pub fn create_text_generation_llama_context(&self, n_seq_max: u32) -> Result<LlamaContext<'_>> {
        let llama_context_params = self
            .llama_context_settings(n_seq_max)?
            .into_llama_context_params();

        Ok(self
            .loaded_llama_model
            .create_llama_context(&self.inference_runtime_context, llama_context_params)?)
    }

    fn llama_context_settings(&self, n_seq_max: u32) -> Result<LlamaContextSettings<'_>> {
        Ok(LlamaContextSettings {
            model_runtime_parameters: &self.model_runtime_parameters,
            n_seq_max,
            thread_count: inference_thread_count()?,
        })
    }
}
