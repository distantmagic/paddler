package loadbalancer

import "github.com/distantmagic/paddler/llamacpp"

type StatsdReporterVoid struct{}

func (self *StatsdReporterVoid) ReportAggregatedHealthStatus(healthStatus *llamacpp.LlamaCppHealthStatus) error {
	return nil
}
