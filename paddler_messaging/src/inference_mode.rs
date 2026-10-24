use serde::Deserialize;
use serde::Serialize;

use crate::inference_capability::InferenceCapability;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum InferenceMode {
    #[default]
    TextGenerationAndEmbeddings,
    TextGenerationEmbeddingsAndDecisions,
}

impl InferenceMode {
    #[must_use]
    pub const fn serves(&self, inference_capability: InferenceCapability) -> bool {
        match inference_capability {
            InferenceCapability::Decisions => match self {
                Self::TextGenerationAndEmbeddings => false,
                Self::TextGenerationEmbeddingsAndDecisions => true,
            },
            InferenceCapability::Embeddings | InferenceCapability::TextGeneration => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InferenceMode;
    use crate::inference_capability::InferenceCapability;

    #[test]
    fn only_the_mode_with_decisions_serves_decisions() {
        assert!(!InferenceMode::TextGenerationAndEmbeddings.serves(InferenceCapability::Decisions));
        assert!(
            InferenceMode::TextGenerationEmbeddingsAndDecisions
                .serves(InferenceCapability::Decisions)
        );
    }

    #[test]
    fn every_mode_serves_text_generation_and_embeddings() {
        for inference_mode in [
            InferenceMode::TextGenerationAndEmbeddings,
            InferenceMode::TextGenerationEmbeddingsAndDecisions,
        ] {
            assert!(inference_mode.serves(InferenceCapability::Embeddings));
            assert!(inference_mode.serves(InferenceCapability::TextGeneration));
        }
    }
}
