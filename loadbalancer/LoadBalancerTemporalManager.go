package loadbalancer

import (
	"time"

	"github.com/hashicorp/go-hclog"
)

type LoadBalancerTemporalManager struct {
	BufferedRequestsStats         *BufferedRequestsStats
	LlamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate
	LoadBalancerTargetCollection  *LoadBalancerTargetCollection
	Logger                        hclog.Logger
	StatsdReporter                StatsdReporterInterface
}

func (self *LoadBalancerTemporalManager) OnApplicationTick() {
	self.ReduceTargetCollectionRemainingTicks()
	self.ReportStats()
}

func (self *LoadBalancerTemporalManager) ReduceTargetCollectionRemainingTicks() {
	var aggregatedSlotsIdle int
	var aggregatedSlotsProcessing int

	targetsMutexToken := self.LoadBalancerTargetCollection.TargetsRBMutex.RLock()

	for element := self.LoadBalancerTargetCollection.Targets.Front(); element != nil; element = element.Next() {
		if element.Value == nil {
			continue
		}

		target := element.Value.(*LlamaCppTarget)
		target.DecrementRemainingTicks()

		if !target.HasRemainingTicks() {
			defer self.LoadBalancerTargetCollection.RemoveTarget(target)
		}

		slotsIdle, slotsProcessing := target.GetSlotsStatus()

		aggregatedSlotsIdle += slotsIdle
		aggregatedSlotsProcessing += slotsProcessing
	}

	self.LoadBalancerTargetCollection.TargetsRBMutex.RUnlock(targetsMutexToken)

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
		self.Logger.Error(
			"error reporting aggregated health status",
			"error", err,
		)
	}
}

func (self *LoadBalancerTemporalManager) RunTickerInterval() {
	ticker := time.NewTicker(time.Second * 1)

	for range ticker.C {
		go self.OnApplicationTick()
	}
}
