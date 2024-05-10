package loadbalancer

import "github.com/distantmagic/paddler/llamacpp"

type LlamaCppTarget struct {
	LlamaCppConfiguration *llamacpp.LlamaCppConfiguration
	LlamaCppClient        *llamacpp.LlamaCppClient
	LlamaCppHealthStatus  *llamacpp.LlamaCppHealthStatus
}

func (self *LlamaCppTarget) Less(other *LlamaCppTarget) bool {
	return self.LlamaCppHealthStatus.Less(other.LlamaCppHealthStatus)
}
