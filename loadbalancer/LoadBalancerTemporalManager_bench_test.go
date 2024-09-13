package loadbalancer

import (
	"net/http"
	"testing"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/hashicorp/go-hclog"
)

func BenchmarkLoadBalancerTemporalManager(b *testing.B) {
	logger := hclog.NewNullLogger()

	llamaCppHealthStatusAggregate := &LlamaCppHealthStatusAggregate{
		AggregatedHealthStatus: &llamacpp.LlamaCppSlotsAggregatedStatus{
			Status: llamacpp.Ok,
		},
	}

	loadBalancerTargetCollection := NewLoadBalancerTargetCollection(llamaCppHealthStatusAggregate)

	loadBalancerTargetRegistrar := &LoadBalancerTargetRegistrar{
		HttpClient:                   http.DefaultClient,
		LoadBalancerConfiguration:    &LoadBalancerConfiguration{},
		LoadBalancerTargetCollection: loadBalancerTargetCollection,
		Logger:                       hclog.NewNullLogger(),
	}

	loadBalancerTemporalManager := &LoadBalancerTemporalManager{
		BufferedRequestsStats:         &BufferedRequestsStats{},
		LlamaCppHealthStatusAggregate: llamaCppHealthStatusAggregate,
		LoadBalancerTargetCollection:  loadBalancerTargetCollection,
		Logger:                        logger,
		StatsdReporter:                &StatsdReporterVoid{},
	}

	target := &LlamaCppTargetConfiguration{
		Id: "target1",
		LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{
				Host:   "127.0.0.1",
				Port:   8081,
				Scheme: "http",
			},
		},
	}

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			loadBalancerTargetRegistrar.RegisterOrUpdateTarget(
				target,
				&llamacpp.LlamaCppSlotsAggregatedStatus{
					SlotsIdle:       8,
					SlotsProcessing: 0,
					Status:          llamacpp.Ok,
					Error:           nil,
				},
			)

			loadBalancerTemporalManager.OnApplicationTick()
		}
	})
}
