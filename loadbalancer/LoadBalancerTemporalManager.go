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

	targetsMutexToken := self.LoadBalancerTargetCollection.RBMutex.RLock()

	for _, target := range self.LoadBalancerTargetCollection.Targets {
		target.DecrementRemainingTicks()

		if !target.HasRemainingTicks() {
			defer self.LoadBalancerTargetCollection.RemoveTarget(target)
		}

		slotsIdle, slotsProcessing := target.GetSlotsStatus()

		aggregatedSlotsIdle += slotsIdle
		aggregatedSlotsProcessing += slotsProcessing
	}

	self.LoadBalancerTargetCollection.RBMutex.RUnlock(targetsMutexToken)

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
