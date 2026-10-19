use std::io;
use std::num::TryFromIntError;
use std::path::PathBuf;

use llama_cpp_bindings::LlamaCppError;
use llama_cpp_bindings::error::LlamaContextLoadError;
use llama_cpp_bindings::error::LlamaModelLoadError;
use llama_cpp_bindings::error::MetaValError;

use paddler_messaging::inference_capability::InferenceCapability;

#[derive(Debug, thiserror::Error)]
pub enum AgentRuntimeError {
    #[error("unable to determine how many threads this machine runs in parallel")]
    AvailableParallelismUnknown(#[source] io::Error),

    #[error("unable to initialize the llama.cpp backend")]
    BackendInitializationFailed(#[source] LlamaCppError),

    #[error("unable to create the llama.cpp context")]
    ContextCreationFailed(#[source] LlamaContextLoadError),

    #[error("a {inference_capability:?} context is already resident in VRAM")]
    LlamaContextAlreadyResident {
        inference_capability: InferenceCapability,
    },

    #[error("unable to load the model from {model_path}")]
    ModelLoadFailed {
        model_path: PathBuf,
        #[source]
        source: LlamaModelLoadError,
    },

    #[error("unable to read the metadata of the loaded model")]
    ModelMetadataUnreadable(#[source] MetaValError),

    #[error("model weights are already resident in VRAM")]
    ModelWeightsAlreadyResident,

    #[error("a multimodal projection is already resident in VRAM")]
    MultimodalProjectionAlreadyResident,

    #[error("the machine runs more threads in parallel than llama.cpp can address")]
    ThreadCountOutOfRange(#[source] TryFromIntError),
}
