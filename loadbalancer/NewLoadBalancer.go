package loadbalancer

import (
	"container/list"
	"net/http"

	"github.com/hashicorp/go-hclog"
)

func NewLoadBalancer(
	httpClient *http.Client,
	logger hclog.Logger,
) *LoadBalancer {
	return &LoadBalancer{
		HttpClient: httpClient,
		LoadBalancerTargetCollection: &LoadBalancerTargetCollection{
			elementByTarget:       make(map[*LlamaCppTarget]*list.Element),
			targetByConfiguration: make(map[string]*LlamaCppTarget),
			Targets:               list.New(),
		},
		Logger: logger,
	}
}
