package loadbalancer

import (
	"container/list"
)

func NewLoadBalancerTargetCollection(
	llamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate,
) *LoadBalancerTargetCollection {
	return &LoadBalancerTargetCollection{
		LlamaCppHealthStatusAggregate: llamaCppHealthStatusAggregate,
		Targets:                       list.New(),

		elementByTarget:       make(map[*LlamaCppTarget]*list.Element),
		targetByConfiguration: make(map[string]*LlamaCppTarget),
	}
}
