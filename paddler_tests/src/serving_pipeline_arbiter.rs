use std::mem::discriminant;
use std::num::NonZeroU16;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use paddler_agent::agent_applicable_state_holder::AgentApplicableStateHolder;
use paddler_agent::desired_state_reconciler::DesiredStateReconciler;
use paddler_agent::desired_state_reconciliation::DesiredStateReconciliation;
use paddler_agent::pipeline_arbiter::PipelineArbiter;
use paddler_agent::pipeline_arbiter_params::PipelineArbiterParams;
use paddler_agent_runtime::inference_runtime_context::InferenceRuntimeContext;
use paddler_agent_status::slot_guard::SlotGuard;
use paddler_cache_dir::cache_dir::CacheDir;
use paddler_messaging::agent_desired_state::AgentDesiredState;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;

use crate::agent_inference_runtime_context::agent_inference_runtime_context;
use crate::desired_state_reconciled_discriminant::desired_state_reconciled_discriminant;

pub struct ServingPipelineArbiter {
    pub desired_state_reconciler: DesiredStateReconciler,
    pub inference_runtime_context: InferenceRuntimeContext,
    pub pipeline_arbiter: PipelineArbiter,
}

impl ServingPipelineArbiter {
    #[must_use]
    pub fn new(slots: u16) -> Self {
        let inference_runtime_context = agent_inference_runtime_context(slots);

        Self {
            desired_state_reconciler: DesiredStateReconciler {
                cache_dir: CacheDir::from_process_env(),
                slot_aggregated_status: inference_runtime_context.slot_aggregated_status.clone(),
            },
            pipeline_arbiter: PipelineArbiter::new(PipelineArbiterParams {
                agent_shutdown: CancellationToken::new(),
                inference_runtime_context: inference_runtime_context.clone(),
                served_applicable_state_holder: Arc::new(AgentApplicableStateHolder::default()),
            }),
            inference_runtime_context,
        }
    }

    pub async fn serve(&mut self, desired_state: BalancerDesiredState) {
        let desired_state_reconciliation = self.apply(desired_state).await;

        assert_eq!(
            discriminant(&desired_state_reconciliation),
            desired_state_reconciled_discriminant()
        );

        self.conclude_next_progress().await;
        self.conclude_next_progress().await;
    }

    pub async fn conclude_next_progress(&mut self) {
        let serving_startup_progress = self.pipeline_arbiter.next_finished_startup().await;

        self.pipeline_arbiter.conclude(serving_startup_progress);
    }

    pub async fn apply(
        &mut self,
        desired_state: BalancerDesiredState,
    ) -> DesiredStateReconciliation {
        let desired_state_reconciliation = self
            .desired_state_reconciler
            .reconcile(
                &CancellationToken::new(),
                AgentDesiredState::from(desired_state),
            )
            .await;

        if let DesiredStateReconciliation::Reconciled(agent_applicable_state) =
            &desired_state_reconciliation
        {
            self.pipeline_arbiter.apply(*agent_applicable_state.clone());
        }

        desired_state_reconciliation
    }

    #[must_use]
    pub fn slot_guard(&self, reserved_slots: NonZeroU16) -> SlotGuard {
        SlotGuard::new(
            self.inference_runtime_context
                .slot_aggregated_status
                .clone(),
            reserved_slots,
        )
    }

    pub async fn shut_down(self) {
        self.pipeline_arbiter.shut_down().await;
    }
}
