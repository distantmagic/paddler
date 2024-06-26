package loadbalancer

import (
	"github.com/distantmagic/paddler/llamacpp"
)

type LlamaCppHealthStatusAggregate struct {
	AggregatedHealthStatus *llamacpp.LlamaCppHealthStatus
}

func (self *LlamaCppHealthStatusAggregate) AddSlotsFrom(llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus) {
	self.IncreaseBy(
		llamaCppHealthStatus.SlotsIdle,
		llamaCppHealthStatus.SlotsProcessing,
	)
}

func (self *LlamaCppHealthStatusAggregate) IncreaseBy(slotsIdle int, slotsProcessing int) {
	self.AggregatedHealthStatus.SlotsIdle += slotsIdle
	self.AggregatedHealthStatus.SlotsProcessing += slotsProcessing
}

func (self *LlamaCppHealthStatusAggregate) RemoveSlotsFrom(llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus) {
	self.IncreaseBy(
		-1*llamaCppHealthStatus.SlotsIdle,
		-1*llamaCppHealthStatus.SlotsProcessing,
	)
}

func (self *LlamaCppHealthStatusAggregate) SetTo(slotsIdle int, slotsProcessing int) {
	self.AggregatedHealthStatus.SlotsIdle = slotsIdle
	self.AggregatedHealthStatus.SlotsProcessing = slotsProcessing
}

func (self *LlamaCppHealthStatusAggregate) UseSlot() {
	self.IncreaseBy(-1, -1)
}
