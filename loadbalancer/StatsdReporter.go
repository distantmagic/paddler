package loadbalancer

import (
	"github.com/distantmagic/paddler/llamacpp"
	statsd "github.com/smira/go-statsd"
)

type StatsdReporter struct {
	StatsdClient statsd.Client
}

func (self *StatsdReporter) ReportAggregatedHealthStatus(healthStatus *llamacpp.LlamaCppHealthStatus) error {
	self.StatsdClient.Gauge("slots_idle", int64(healthStatus.SlotsIdle))
	self.StatsdClient.Gauge("slots_processing", int64(healthStatus.SlotsProcessing))

	return nil
}
