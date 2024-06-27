package loadbalancer

import (
	"time"

	"github.com/distantmagic/paddler/goroutine"
)

type LoadBalancerTemporalManager struct {
	BufferedRequestsStats         *BufferedRequestsStats
	LlamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate
	LoadBalancerTargetCollection  *LoadBalancerTargetCollection
	ServerEventsChannel           chan<- goroutine.ResultMessage
	StatsdReporter                StatsdReporterInterface
}

func (self *LoadBalancerTemporalManager) OnApplicationTick() {
	self.ReduceTargetCollectionRemainingTicks()
	self.ReportStats()
}

func (self *LoadBalancerTemporalManager) ReduceTargetCollectionRemainingTicks() {
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
	err := self.StatsdReporter.ReportAggregatedHealthStatus(
		self.BufferedRequestsStats,
		self.LlamaCppHealthStatusAggregate,
	)

	if err != nil {
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Comment: "error reporting aggregated health status",
			Error:   err,
		}
	}
}

func (self *LoadBalancerTemporalManager) RunTickerInterval() {
	ticker := time.NewTicker(time.Second * 1)

	for range ticker.C {
		go self.OnApplicationTick()
	}
}
