use paddler_messaging::agent_runtime_status::AgentRuntimeStatus;
use paddler_messaging::agent_status::AgentStatus;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_messaging::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;

pub fn serving_status_snapshot(
    inference_mode: InferenceMode,
    slots_total: u64,
    version: u64,
) -> SlotAggregatedStatusSnapshot {
    SlotAggregatedStatusSnapshot {
        status: AgentStatus {
            runtime: AgentRuntimeStatus::Serving {
                inference_mode,
                slots_total,
            },
            ..AgentStatus::default()
        },
        version,
    }
}
