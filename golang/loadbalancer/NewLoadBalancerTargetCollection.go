package loadbalancer

import (
	"github.com/puzpuzpuz/xsync/v3"
)

func NewLoadBalancerTargetCollection(
	llamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate,
) *LoadBalancerTargetCollection {
	return &LoadBalancerTargetCollection{
		LlamaCppHealthStatusAggregate: llamaCppHealthStatusAggregate,
		Targets:                       make([]*LlamaCppTarget, 0),

		targetById: xsync.NewMapOf[string, *LlamaCppTarget](),
	}
}
