package loadbalancer

import (
	"net/http/httputil"
	"sync"
	"time"

	"github.com/distantmagic/paddler/llamacpp"
)

type LlamaCppTarget struct {
	LastUpdate                  time.Time
	LlamaCppTargetConfiguration *LlamaCppTargetConfiguration
	LlamaCppClient              *llamacpp.LlamaCppClient
	LlamaCppHealthStatus        *llamacpp.LlamaCppHealthStatus
	RemainingTicksUntilRemoved  int
	RWMutex                     sync.RWMutex
	TotalUpdates                int
	ReverseProxy                *httputil.ReverseProxy
}

func (self *LlamaCppTarget) DecrementRemainingTicks() {
	self.RWMutex.Lock()
	defer self.RWMutex.Unlock()

	self.RemainingTicksUntilRemoved -= 1
}

func (self *LlamaCppTarget) HasLessSlotsThan(other *LlamaCppTarget) bool {
	return self.LlamaCppHealthStatus.SlotsIdle < other.LlamaCppHealthStatus.SlotsIdle
}

func (self *LlamaCppTarget) SetTickStatus(
	lastUpdate time.Time,
	remainingTicksUntilRemoved int,
) {
	self.RWMutex.Lock()
	defer self.RWMutex.Unlock()

	self.LastUpdate = lastUpdate
	self.RemainingTicksUntilRemoved = remainingTicksUntilRemoved
	self.TotalUpdates += 1
}
