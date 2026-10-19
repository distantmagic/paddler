use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use paddler_messaging::inference_capability::InferenceCapability;
use paddler_messaging::inference_mode::InferenceMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClusterDoesNotServeInferenceCapability {
    pub requested_inference_capability: InferenceCapability,
    pub served_inference_mode: InferenceMode,
}

impl Display for ClusterDoesNotServeInferenceCapability {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            requested_inference_capability,
            served_inference_mode,
        } = self;

        write!(
            formatter,
            "The cluster serves {served_inference_mode:?}, which does not include {requested_inference_capability:?}"
        )
    }
}

#[cfg(test)]
mod tests {
    use paddler_messaging::inference_capability::InferenceCapability;
    use paddler_messaging::inference_mode::InferenceMode;

    use super::ClusterDoesNotServeInferenceCapability;

    #[test]
    fn names_the_served_mode_and_the_capability_it_lacks() {
        assert_eq!(
            ClusterDoesNotServeInferenceCapability {
                requested_inference_capability: InferenceCapability::Decisions,
                served_inference_mode: InferenceMode::TextGenerationAndEmbeddings,
            }
            .to_string(),
            "The cluster serves TextGenerationAndEmbeddings, which does not include Decisions"
        );
    }
}
