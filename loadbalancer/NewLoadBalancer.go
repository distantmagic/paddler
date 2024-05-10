package loadbalancer

import (
	"github.com/emirpasic/gods/v2/trees/binaryheap"
	"github.com/hashicorp/go-hclog"
)

func NewLoadBalancer(
	logger hclog.Logger,
) *LoadBalancer {
	return &LoadBalancer{
		Logger:  logger,
		targets: binaryheap.NewWith[*LlamaCppTarget](LlamaCppTargetComparator),
	}
}
