package loadbalancer

import (
	"sync"

	"github.com/distantmagic/paddler/goroutine"
)

type LoadBalancerTemporalManager struct {
	LlamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate
	LoadBalancerTargetCollection  *LoadBalancerTargetCollection
	ServerEventsChannel           chan<- goroutine.ResultMessage
	StatsdReporter                StatsdReporterInterface

	mu sync.Mutex
}

func (self *LoadBalancerTemporalManager) OnApplicationTick() {
	self.ReduceTargetCollectionRemainingTicks()
	self.ReportStats()
}

func (self *LoadBalancerTemporalManager) ReduceTargetCollectionRemainingTicks() {
	self.mu.Lock()
	defer self.mu.Unlock()

	var aggregatedSlotsIdle int
	var aggregatedSlotsProcessing int

	for element := self.LoadBalancerTargetCollection.Targets.Front(); element != nil; element = element.Next() {
		target := element.Value.(*LlamaCppTarget)
		target.RemainingTicksUntilRemoved -= 1

		if target.RemainingTicksUntilRemoved < 1 {
			defer self.LoadBalancerTargetCollection.RemoveTarget(target)
		}

		aggregatedSlotsIdle += target.LlamaCppHealthStatus.SlotsIdle
		aggregatedSlotsProcessing += target.LlamaCppHealthStatus.SlotsProcessing
	}

	self.LlamaCppHealthStatusAggregate.SetTo(
		aggregatedSlotsIdle,
		aggregatedSlotsProcessing,
	)
}

func (self *LoadBalancerTemporalManager) ReportStats() {
	err := self.StatsdReporter.ReportAggregatedHealthStatus(self.LlamaCppHealthStatusAggregate)

	if err != nil {
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Comment: "error reporting aggregated health status",
			Error:   err,
		}
	}
}
