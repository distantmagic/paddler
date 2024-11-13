package loadbalancer

type StatsdReporterInterface interface {
	ReportAggregatedHealthStatus(
		bufferedRequestsStats *BufferedRequestsStats,
		llamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate,
	) error
}
