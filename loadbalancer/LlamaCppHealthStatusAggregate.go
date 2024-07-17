package loadbalancer

import (
	"sync"

	"github.com/distantmagic/paddler/llamacpp"
)

type LlamaCppHealthStatusAggregate struct {
	AggregatedHealthStatus *llamacpp.LlamaCppHealthStatus
	RWMutex                sync.RWMutex
}

func (self *LlamaCppHealthStatusAggregate) AddSlotsFrom(llamaCppTarget *LlamaCppTarget) {
	llamaCppTarget.RWMutex.RLock()
	defer llamaCppTarget.RWMutex.RUnlock()

	self.IncreaseBy(
		llamaCppTarget.LlamaCppHealthStatus.SlotsIdle,
		llamaCppTarget.LlamaCppHealthStatus.SlotsProcessing,
	)
}

func (self *LlamaCppHealthStatusAggregate) IncreaseBy(slotsIdle int, slotsProcessing int) {
	self.RWMutex.Lock()
	defer self.RWMutex.Unlock()

	self.AggregatedHealthStatus.SlotsIdle += slotsIdle
	self.AggregatedHealthStatus.SlotsProcessing += slotsProcessing
}

func (self *LlamaCppHealthStatusAggregate) RemoveSlotsFrom(llamaCppTarget *LlamaCppTarget) {
	llamaCppTarget.RWMutex.RLock()
	defer llamaCppTarget.RWMutex.RUnlock()

	self.IncreaseBy(
		-1*llamaCppTarget.LlamaCppHealthStatus.SlotsIdle,
		-1*llamaCppTarget.LlamaCppHealthStatus.SlotsProcessing,
	)
}

func (self *LlamaCppHealthStatusAggregate) SetTo(slotsIdle int, slotsProcessing int) {
	self.RWMutex.Lock()
	defer self.RWMutex.Unlock()

	self.AggregatedHealthStatus.SlotsIdle = slotsIdle
	self.AggregatedHealthStatus.SlotsProcessing = slotsProcessing
}

func (self *LlamaCppHealthStatusAggregate) UseSlot() {
	self.IncreaseBy(-1, -1)
}
