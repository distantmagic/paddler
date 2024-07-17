package loadbalancer

import "github.com/distantmagic/paddler/llamacpp"

type LlamaCppTargetConfiguration struct {
	Id                    string
	LlamaCppConfiguration *llamacpp.LlamaCppConfiguration `json:"llama_cpp_configuration"`
}
