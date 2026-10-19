use paddler_agent_runtime::loaded_llama_model::LoadedLlamaModel;
use paddler_agent_runtime::shuts_down_unless_shared::ShutsDownUnlessShared;

use crate::capability_plan::CapabilityPlan;
use crate::inference_capabilities::InferenceCapabilities;
use crate::model_serving_settings::ModelServingSettings;
use crate::served_decisions::ServedDecisions;
use crate::served_embeddings::ServedEmbeddings;
use crate::served_text_generation::ServedTextGeneration;

#[derive(Clone)]
pub struct LoadedModelServingPlan<TDecisionsPlan = CapabilityPlan<ServedDecisions>> {
    pub capability_plans: InferenceCapabilities<
        TDecisionsPlan,
        CapabilityPlan<ServedEmbeddings>,
        CapabilityPlan<ServedTextGeneration>,
    >,
    pub loaded_llama_model: LoadedLlamaModel,
    pub model_serving_settings: ModelServingSettings,
}

impl<TDecisionsPlan> LoadedModelServingPlan<TDecisionsPlan>
where
    TDecisionsPlan: ShutsDownUnlessShared + Send,
{
    pub async fn release(self) {
        let Self {
            capability_plans,
            loaded_llama_model,
            ..
        } = self;

        capability_plans.shut_down_unless_shared().await;
        loaded_llama_model.release().await;
    }
}
