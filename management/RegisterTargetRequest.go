package management

import (
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/loadbalancer"
)

type RegisterTargetRequest struct {
	LlamaCppSlotsAggregatedStatus *llamacpp.LlamaCppSlotsAggregatedStatus   `json:"llama_cpp_slots_aggregated_status"`
	LlamaCppTargetConfiguration   *loadbalancer.LlamaCppTargetConfiguration `json:"llama_cpp_target_configuration"`
	PaddlerAgentName              string                                    `json:"paddler_agent_name"`
}
