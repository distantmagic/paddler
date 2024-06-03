package loadbalancer

import "github.com/distantmagic/paddler/llamacpp"

type StatsdReporterInterface interface {
	ReportAggregatedHealthStatus(healthStatus *llamacpp.LlamaCppHealthStatus) error
}
