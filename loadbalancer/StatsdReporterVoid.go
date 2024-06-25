package loadbalancer

type StatsdReporterVoid struct{}

func (self *StatsdReporterVoid) ReportAggregatedHealthStatus(llamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate) error {
	return nil
}
