package loadbalancer

import (
	"container/heap"

	"github.com/hashicorp/go-hclog"
)

func NewLoadBalancer(
	logger hclog.Logger,
) *LoadBalancer {
	targetHeap := &LlamaCppTargetHeap{}

	heap.Init(targetHeap)

	return &LoadBalancer{
		Logger:  logger,
		targets: targetHeap,
	}
}
