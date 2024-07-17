package loadbalancer

import (
	"container/list"

	"github.com/puzpuzpuz/xsync/v3"
)

func NewLoadBalancerTargetCollection(
	llamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate,
) *LoadBalancerTargetCollection {
	return &LoadBalancerTargetCollection{
		LlamaCppHealthStatusAggregate: llamaCppHealthStatusAggregate,
		Targets:                       list.New(),

		elementByTarget:       xsync.NewMapOf[*LlamaCppTarget, *list.Element](),
		targetByConfiguration: xsync.NewMapOf[string, *LlamaCppTarget](),
	}
}
