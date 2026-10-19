use tokio_util::sync::CancellationToken;

use paddler_agent_runtime::inference_runtime_context::InferenceRuntimeContext;

use crate::agent_applicable_state::AgentApplicableState;
use crate::capability_plan::CapabilityPlan;
use crate::capability_starter::CapabilityStarter;
use crate::capability_startup_interruption::CapabilityStartupInterruption;
use crate::inference_capabilities::InferenceCapabilities;
use crate::loaded_model_serving_plan::LoadedModelServingPlan;
use crate::model_serving::ModelServing;
use crate::model_serving_settings::ModelServingSettings;
use crate::prepared_decisions_plan::PreparedDecisionsPlan;
use crate::serving_startup_failure::ServingStartupFailure;
use crate::started_serving::StartedServing;

#[derive(Clone)]
pub enum ServingStartupPlan {
    LoadModel(ModelServingSettings),
    NothingToServe,
    OnLoadedModel(LoadedModelServingPlan),
}

impl ServingStartupPlan {
    #[must_use]
    pub fn new(
        model_serving: &ModelServing,
        agent_applicable_state: &AgentApplicableState,
    ) -> Self {
        match agent_applicable_state {
            AgentApplicableState::NotConfigured | AgentApplicableState::WithoutModel { .. } => {
                Self::NothingToServe
            }
            AgentApplicableState::WithModel(desired_settings) => match model_serving {
                ModelServing::Serving(served_model)
                    if served_model.model_serving_settings.llama_model_settings()
                        == desired_settings.llama_model_settings() =>
                {
                    Self::OnLoadedModel(served_model.plan_restart(desired_settings.clone()))
                }
                ModelServing::Idle | ModelServing::Serving(_) => {
                    Self::LoadModel(desired_settings.clone())
                }
            },
        }
    }

    pub async fn release(self) {
        match self {
            Self::LoadModel(_) | Self::NothingToServe => {}
            Self::OnLoadedModel(loaded_model_serving_plan) => {
                loaded_model_serving_plan.release().await;
            }
        }
    }

    pub async fn start(
        self,
        cancellation_token: &CancellationToken,
        inference_runtime_context: &InferenceRuntimeContext,
    ) -> Result<StartedServing, ServingStartupFailure> {
        if cancellation_token.is_cancelled() {
            self.release().await;

            return Ok(StartedServing::Cancelled);
        }

        let capability_starter = CapabilityStarter {
            cancellation_token,
            inference_runtime_context,
        };

        match self {
            Self::LoadModel(model_serving_settings) => {
                let prepared_decisions_plan = match capability_starter
                    .prepare_decisions(
                        CapabilityPlan::Start,
                        &model_serving_settings.capabilities.decisions,
                    )
                    .await
                {
                    Ok(prepared_decisions_plan) => prepared_decisions_plan,
                    Err(interruption) => {
                        return Self::LoadModel(model_serving_settings)
                            .after_interruption(interruption)
                            .await;
                    }
                };
                let loaded_llama_model = match capability_starter
                    .load_model(model_serving_settings.llama_model_settings())
                    .await
                {
                    Ok(loaded_llama_model) => loaded_llama_model,
                    Err(interruption) => {
                        return Self::LoadModel(model_serving_settings)
                            .after_interruption(interruption)
                            .await;
                    }
                };
                let retry_plan = Self::OnLoadedModel(LoadedModelServingPlan {
                    capability_plans: InferenceCapabilities {
                        decisions: CapabilityPlan::Start,
                        embeddings: CapabilityPlan::Start,
                        text_generation: CapabilityPlan::Start,
                    },
                    loaded_llama_model: loaded_llama_model.clone(),
                    model_serving_settings: model_serving_settings.clone(),
                });

                retry_plan
                    .serve_or_retry(
                        &capability_starter,
                        LoadedModelServingPlan {
                            capability_plans: InferenceCapabilities {
                                decisions: prepared_decisions_plan,
                                embeddings: CapabilityPlan::Start,
                                text_generation: CapabilityPlan::Start,
                            },
                            loaded_llama_model,
                            model_serving_settings,
                        },
                    )
                    .await
            }
            Self::NothingToServe => Ok(StartedServing::NothingToServe),
            Self::OnLoadedModel(loaded_model_serving_plan) => {
                let prepared_decisions_plan = match capability_starter
                    .prepare_decisions(
                        loaded_model_serving_plan.capability_plans.decisions.clone(),
                        &loaded_model_serving_plan
                            .model_serving_settings
                            .capabilities
                            .decisions,
                    )
                    .await
                {
                    Ok(prepared_decisions_plan) => prepared_decisions_plan,
                    Err(interruption) => {
                        return Self::OnLoadedModel(loaded_model_serving_plan)
                            .after_interruption(interruption)
                            .await;
                    }
                };
                let retry_plan = Self::OnLoadedModel(loaded_model_serving_plan.clone());
                let LoadedModelServingPlan {
                    capability_plans:
                        InferenceCapabilities {
                            embeddings,
                            text_generation,
                            ..
                        },
                    loaded_llama_model,
                    model_serving_settings,
                } = loaded_model_serving_plan;

                retry_plan
                    .serve_or_retry(
                        &capability_starter,
                        LoadedModelServingPlan {
                            capability_plans: InferenceCapabilities {
                                decisions: prepared_decisions_plan,
                                embeddings,
                                text_generation,
                            },
                            loaded_llama_model,
                            model_serving_settings,
                        },
                    )
                    .await
            }
        }
    }

    async fn after_interruption(
        self,
        interruption: CapabilityStartupInterruption,
    ) -> Result<StartedServing, ServingStartupFailure> {
        match interruption {
            CapabilityStartupInterruption::Cancelled => {
                self.release().await;

                Ok(StartedServing::Cancelled)
            }
            CapabilityStartupInterruption::Failed(error) => Err(ServingStartupFailure {
                error,
                retry_plan: self,
            }),
        }
    }

    async fn serve_or_retry(
        self,
        capability_starter: &CapabilityStarter<'_>,
        loaded_model_serving_plan: LoadedModelServingPlan<PreparedDecisionsPlan>,
    ) -> Result<StartedServing, ServingStartupFailure> {
        match capability_starter.serve(loaded_model_serving_plan).await {
            Ok(served_model) => Ok(StartedServing::Serving(Box::new(served_model))),
            Err(interruption) => self.after_interruption(interruption).await,
        }
    }
}
