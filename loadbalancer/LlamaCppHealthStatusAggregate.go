package loadbalancer

import (
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/puzpuzpuz/xsync/v3"
)

type LlamaCppHealthStatusAggregate struct {
	AggregatedHealthStatus *llamacpp.LlamaCppSlotsAggregatedStatus
	RBMutex                xsync.RBMutex
}

func (self *LlamaCppHealthStatusAggregate) AddSlotsFrom(llamaCppTarget *LlamaCppTarget) {
	mutexToken := llamaCppTarget.RBMutex.RLock()
	defer llamaCppTarget.RBMutex.RUnlock(mutexToken)

	self.IncreaseBy(
		llamaCppTarget.LlamaCppSlotsAggregatedStatus.SlotsIdle,
		llamaCppTarget.LlamaCppSlotsAggregatedStatus.SlotsProcessing,
	)
}

func (self *LlamaCppHealthStatusAggregate) IncreaseBy(slotsIdle int, slotsProcessing int) {
	self.RBMutex.Lock()
	defer self.RBMutex.Unlock()

	self.AggregatedHealthStatus.SlotsIdle += slotsIdle
	self.AggregatedHealthStatus.SlotsProcessing += slotsProcessing
}

func (self *LlamaCppHealthStatusAggregate) RemoveSlotsFrom(llamaCppTarget *LlamaCppTarget) {
	mutexToken := llamaCppTarget.RBMutex.RLock()
	defer llamaCppTarget.RBMutex.RUnlock(mutexToken)

	self.IncreaseBy(
		-1*llamaCppTarget.LlamaCppSlotsAggregatedStatus.SlotsIdle,
		-1*llamaCppTarget.LlamaCppSlotsAggregatedStatus.SlotsProcessing,
	)
}

func (self *LlamaCppHealthStatusAggregate) SetTo(slotsIdle int, slotsProcessing int) {
	self.RBMutex.Lock()
	defer self.RBMutex.Unlock()

	self.AggregatedHealthStatus.SlotsIdle = slotsIdle
	self.AggregatedHealthStatus.SlotsProcessing = slotsProcessing
}

func (self *LlamaCppHealthStatusAggregate) UseSlot() {
	self.IncreaseBy(-1, 1)
}
