package loadbalancer

type StatsdReporterInterface interface {
	ReportAggregatedHealthStatus(llamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate) error
}
