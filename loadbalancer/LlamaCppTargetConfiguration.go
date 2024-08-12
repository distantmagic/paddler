package loadbalancer

import "github.com/distantmagic/paddler/llamacpp"

type LlamaCppTargetConfiguration struct {
	Id                    string
	Name                  string
	LlamaCppConfiguration *llamacpp.LlamaCppConfiguration `json:"llama_cpp_configuration"`
}
