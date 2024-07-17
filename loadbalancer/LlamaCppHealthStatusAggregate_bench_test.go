package loadbalancer

import (
	"testing"

	"github.com/distantmagic/paddler/llamacpp"
)

func BenchmarkLlamaCppHealthStatusAggregate(b *testing.B) {
	llamaCppHealthStatusAggregate := &LlamaCppHealthStatusAggregate{
		AggregatedHealthStatus: &llamacpp.LlamaCppHealthStatus{
			Status: llamacpp.Ok,
		},
	}

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			llamaCppHealthStatusAggregate.IncreaseBy(1, 1)
		}
	})
}
