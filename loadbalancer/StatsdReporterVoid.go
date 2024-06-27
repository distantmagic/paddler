package loadbalancer

type StatsdReporterVoid struct{}

func (self *StatsdReporterVoid) ReportAggregatedHealthStatus(
	bufferedRequestsStats *BufferedRequestsStats,
	llamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate,
) error {
	return nil
}
