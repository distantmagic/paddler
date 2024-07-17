package loadbalancer

import (
	"net/http"
	"testing"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/hashicorp/go-hclog"
)

func BenchmarkLoadBalancerTargetCollection(b *testing.B) {
	llamaCppHealthStatusAggregate := &LlamaCppHealthStatusAggregate{
		AggregatedHealthStatus: &llamacpp.LlamaCppHealthStatus{
			Status: llamacpp.Ok,
		},
	}

	loadBalancerTargetRegistrar := &LoadBalancerTargetRegistrar{
		HttpClient:                   http.DefaultClient,
		LoadBalancerTargetCollection: NewLoadBalancerTargetCollection(llamaCppHealthStatusAggregate),
		Logger:                       hclog.NewNullLogger(),
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
				&llamacpp.LlamaCppHealthStatus{
					Status:          llamacpp.Ok,
					SlotsIdle:       8,
					SlotsProcessing: 0,
					Error:           nil,
				},
			)
		}
	})
}
