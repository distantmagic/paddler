package loadbalancer

import (
	statsd "github.com/smira/go-statsd"
)

type StatsdReporter struct {
	StatsdClient statsd.Client
}

func (self *StatsdReporter) ReportAggregatedHealthStatus(llamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate) error {
	self.StatsdClient.Gauge("slots_idle", int64(llamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsIdle))
	self.StatsdClient.Gauge("slots_processing", int64(llamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsProcessing))

	return nil
}
