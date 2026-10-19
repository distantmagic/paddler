use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use paddler_agent_decision::decision_pipeline::DecisionPipeline;
use paddler_agent_decision::decision_prerequisites::DecisionPrerequisites;
use paddler_agent_decision::general_architecture_metadata::GeneralArchitectureMetadata;
use paddler_agent_embeddings::embedding_pipeline::EmbeddingPipeline;
use paddler_agent_runtime::inference_runtime_context::InferenceRuntimeContext;
use paddler_agent_runtime::llama_model_settings::LlamaModelSettings;
use paddler_agent_runtime::loaded_llama_model::LoadedLlamaModel;
use paddler_agent_runtime::shuts_down_unless_shared::ShutsDownUnlessShared as _;
use paddler_agent_runtime::spawn_blocking_resuming_panic::spawn_blocking_resuming_panic;
use paddler_agent_runtime::startup_outcome::StartupOutcome;
use paddler_agent_text_generation::llama_cpp_media_marker::LlamaCppMediaMarker;
use paddler_agent_text_generation::text_generation_pipeline::TextGenerationPipeline;
use paddler_inference_parameters::model_runtime_parameters::ModelRuntimeParameters;

use crate::agent_error::AgentError;
use crate::applicable_decisions::ApplicableDecisions;
use crate::capability_plan::CapabilityPlan;
use crate::capability_startup_interruption::CapabilityStartupInterruption;
use crate::inference_capabilities::InferenceCapabilities;
use crate::loaded_model_serving_plan::LoadedModelServingPlan;
use crate::model_serving_settings::ModelServingSettings;
use crate::prepared_decisions::PreparedDecisions;
use crate::prepared_decisions_plan::PreparedDecisionsPlan;
use crate::served_capabilities::ServedCapabilities;
use crate::served_decisions::ServedDecisions;
use crate::served_embeddings::ServedEmbeddings;
use crate::served_model::ServedModel;
use crate::served_text_generation::ServedTextGeneration;

trait IntoEstablishedCapability<TStarted> {
    fn into_established_capability(self) -> Result<TStarted, CapabilityStartupInterruption>;
}

impl<TStarted> IntoEstablishedCapability<TStarted> for StartupOutcome<TStarted> {
    fn into_established_capability(self) -> Result<TStarted, CapabilityStartupInterruption> {
        match self {
            Self::Ready(started) => Ok(started),
            Self::Cancelled => Err(CapabilityStartupInterruption::Cancelled),
        }
    }
}

pub struct CapabilityStarter<'startup> {
    pub cancellation_token: &'startup CancellationToken,
    pub inference_runtime_context: &'startup InferenceRuntimeContext,
}

impl CapabilityStarter<'_> {
    pub async fn prepare_decisions(
        &self,
        decisions_plan: CapabilityPlan<ServedDecisions>,
        applicable_decisions: &ApplicableDecisions,
    ) -> Result<PreparedDecisionsPlan, CapabilityStartupInterruption> {
        match (decisions_plan, applicable_decisions) {
            (CapabilityPlan::Keep(served_decisions), _) => {
                Ok(PreparedDecisionsPlan::Keep(served_decisions))
            }
            (CapabilityPlan::Start, ApplicableDecisions::Disabled) => {
                Ok(PreparedDecisionsPlan::Start(PreparedDecisions::Disabled))
            }
            (CapabilityPlan::Start, ApplicableDecisions::Enabled { pointer_head_path }) => {
                let decision_prerequisites = DecisionPrerequisites {
                    inference_runtime_context: self.inference_runtime_context.clone(),
                    pointer_head_path: pointer_head_path.clone(),
                };

                spawn_blocking_resuming_panic(move || decision_prerequisites.load_pointer_head())
                    .await
                    .map(|loaded_pointer_head| {
                        PreparedDecisionsPlan::Start(PreparedDecisions::Enabled(Box::new(
                            loaded_pointer_head,
                        )))
                    })
                    .map_err(|decision_error| {
                        CapabilityStartupInterruption::Failed(AgentError::from(decision_error))
                    })
            }
        }
    }

    pub async fn load_model(
        &self,
        llama_model_settings: LlamaModelSettings,
    ) -> Result<LoadedLlamaModel, CapabilityStartupInterruption> {
        LoadedLlamaModel::load_unless_shutting_down(
            self.cancellation_token,
            self.inference_runtime_context.clone(),
            llama_model_settings,
        )
        .await
        .map_err(AgentError::from)?
        .into_established_capability()
    }

    pub async fn serve(
        &self,
        loaded_model_serving_plan: LoadedModelServingPlan<PreparedDecisionsPlan>,
    ) -> Result<ServedModel, CapabilityStartupInterruption> {
        let loaded_llama_model = loaded_model_serving_plan.loaded_llama_model.clone();
        let model_serving_settings = loaded_model_serving_plan.model_serving_settings.clone();

        match self.establish(loaded_model_serving_plan).await {
            Ok(capabilities) => Ok(ServedModel {
                capabilities,
                loaded_llama_model,
                model_serving_settings,
            }),
            Err(interruption) => {
                loaded_llama_model.release().await;

                Err(interruption)
            }
        }
    }

    async fn establish(
        &self,
        LoadedModelServingPlan {
            capability_plans:
                InferenceCapabilities {
                    decisions: decisions_plan,
                    embeddings: embeddings_plan,
                    text_generation: text_generation_plan,
                },
            loaded_llama_model,
            model_serving_settings,
        }: LoadedModelServingPlan<PreparedDecisionsPlan>,
    ) -> Result<ServedCapabilities, CapabilityStartupInterruption> {
        let decisions = self
            .establish_decisions(
                &loaded_llama_model,
                &model_serving_settings.model_runtime_parameters,
                decisions_plan,
            )
            .await?;
        let embeddings = match self
            .establish_embeddings(
                &loaded_llama_model,
                &model_serving_settings,
                embeddings_plan,
            )
            .await
        {
            Ok(embeddings) => embeddings,
            Err(interruption) => {
                decisions.shut_down_unless_shared().await;

                return Err(interruption);
            }
        };
        let text_generation = match self
            .establish_text_generation(
                loaded_llama_model,
                &model_serving_settings,
                text_generation_plan,
            )
            .await
        {
            Ok(text_generation) => text_generation,
            Err(interruption) => {
                decisions.shut_down_unless_shared().await;
                embeddings.shut_down_unless_shared().await;

                return Err(interruption);
            }
        };

        Ok(InferenceCapabilities {
            decisions,
            embeddings,
            text_generation,
        })
    }

    async fn establish_embeddings(
        &self,
        loaded_llama_model: &LoadedLlamaModel,
        model_serving_settings: &ModelServingSettings,
        embeddings_plan: CapabilityPlan<ServedEmbeddings>,
    ) -> Result<ServedEmbeddings, CapabilityStartupInterruption> {
        match embeddings_plan {
            CapabilityPlan::Keep(embeddings) => Ok(embeddings),
            CapabilityPlan::Start => EmbeddingPipeline {
                inference_runtime_context: self.inference_runtime_context.clone(),
                loaded_llama_model: loaded_llama_model.clone(),
                model_runtime_parameters: model_serving_settings.model_runtime_parameters.clone(),
                pooling_type: model_serving_settings.capabilities.embeddings.clone(),
            }
            .spawn(self.cancellation_token)
            .await
            .map_err(AgentError::from)?
            .into_established_capability()
            .map(Arc::new),
        }
    }

    async fn establish_text_generation(
        &self,
        loaded_llama_model: LoadedLlamaModel,
        model_serving_settings: &ModelServingSettings,
        text_generation_plan: CapabilityPlan<ServedTextGeneration>,
    ) -> Result<ServedTextGeneration, CapabilityStartupInterruption> {
        match text_generation_plan {
            CapabilityPlan::Keep(text_generation) => Ok(text_generation),
            CapabilityPlan::Start => TextGenerationPipeline {
                inference_runtime_context: self.inference_runtime_context.clone(),
                loaded_llama_model,
                media_marker: LlamaCppMediaMarker,
                model_runtime_parameters: model_serving_settings.model_runtime_parameters.clone(),
                text_generation_settings: model_serving_settings
                    .capabilities
                    .text_generation
                    .clone(),
            }
            .spawn(self.cancellation_token)
            .await
            .map_err(AgentError::from)?
            .into_established_capability()
            .map(Arc::new),
        }
    }

    async fn establish_decisions(
        &self,
        loaded_llama_model: &LoadedLlamaModel,
        model_runtime_parameters: &ModelRuntimeParameters,
        prepared_decisions_plan: PreparedDecisionsPlan,
    ) -> Result<ServedDecisions, CapabilityStartupInterruption> {
        match prepared_decisions_plan {
            PreparedDecisionsPlan::Keep(served_decisions) => Ok(served_decisions),
            PreparedDecisionsPlan::Start(PreparedDecisions::Disabled) => {
                Ok(ServedDecisions::Disabled)
            }
            PreparedDecisionsPlan::Start(PreparedDecisions::Enabled(loaded_pointer_head)) => {
                DecisionPipeline {
                    architecture: GeneralArchitectureMetadata,
                    inference_runtime_context: self.inference_runtime_context.clone(),
                    loaded_llama_model: loaded_llama_model.clone(),
                    loaded_pointer_head: *loaded_pointer_head,
                    model_runtime_parameters: model_runtime_parameters.clone(),
                }
                .spawn(self.cancellation_token)
                .await
                .map_err(AgentError::from)?
                .into_established_capability()
                .map(|decisions| ServedDecisions::Enabled(Arc::new(decisions)))
            }
        }
    }
}
