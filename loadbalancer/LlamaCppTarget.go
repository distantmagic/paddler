package loadbalancer

import (
	"time"

	"github.com/distantmagic/paddler/llamacpp"
)

type LlamaCppTarget struct {
	LastUpdate                  time.Time
	LlamaCppTargetConfiguration *LlamaCppTargetConfiguration
	LlamaCppClient              *llamacpp.LlamaCppClient
	LlamaCppHealthStatus        *llamacpp.LlamaCppHealthStatus
	RemainingTicksUntilRemoved  int
	TotalUpdates                int
}

func (self *LlamaCppTarget) HasLessSlotsThan(other *LlamaCppTarget) bool {
	return self.LlamaCppHealthStatus.SlotsIdle < other.LlamaCppHealthStatus.SlotsIdle
}
