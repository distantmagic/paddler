package management

import (
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/loadbalancer"
)

type RegisterTargetRequest struct {
	LlamaCppHealthStatus        *llamacpp.LlamaCppHealthStatus            `json:"llama_cpp_health_status"`
	LlamaCppTargetConfiguration *loadbalancer.LlamaCppTargetConfiguration `json:"llama_cpp_target_configuration"`
	PaddlerAgentName            string                                    `json:"paddler_agent_name"`
}
