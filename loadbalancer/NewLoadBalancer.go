package loadbalancer

import (
	"container/heap"
	"net/http"

	"github.com/hashicorp/go-hclog"
)

func NewLoadBalancer(
	httpClient *http.Client,
	logger hclog.Logger,
) *LoadBalancer {
	targetHeap := &LlamaCppTargetHeap{}

	heap.Init(targetHeap)

	return &LoadBalancer{
		HttpClient: httpClient,
		Logger:     logger,
		targets:    targetHeap,
	}
}
