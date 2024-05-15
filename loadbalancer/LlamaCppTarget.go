package loadbalancer

import "github.com/distantmagic/paddler/llamacpp"

type LlamaCppTarget struct {
	LlamaCppTargetConfiguration *LlamaCppTargetConfiguration
	LlamaCppClient              *llamacpp.LlamaCppClient
	LlamaCppHealthStatus        *llamacpp.LlamaCppHealthStatus
}

func (self *LlamaCppTarget) HasLessSlotsThan(other *LlamaCppTarget) bool {
	return self.LlamaCppHealthStatus.SlotsIdle < other.LlamaCppHealthStatus.SlotsIdle
}
