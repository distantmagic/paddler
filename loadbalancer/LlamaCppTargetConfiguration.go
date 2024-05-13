package loadbalancer

import "github.com/distantmagic/paddler/llamacpp"

type LlamaCppTargetConfiguration struct {
	LlamaCppConfiguration *llamacpp.LlamaCppConfiguration `json:"llama_cpp_configuration"`
}

func (self *LlamaCppTargetConfiguration) String() string {
	return self.LlamaCppConfiguration.String()
}
