package loadbalancer

import (
	"container/list"
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/hashicorp/go-hclog"
)

func NewLoadBalancer(
	httpClient *http.Client,
	logger hclog.Logger,
	serverEventsChannel chan<- goroutine.ResultMessage,
	statsdReporter StatsdReporterInterface,
) *LoadBalancer {
	loadBalancerTargetCollection := &LoadBalancerTargetCollection{
		AggregatedHealthStatus: &llamacpp.LlamaCppHealthStatus{
			Status: llamacpp.Ok,
		},
		elementByTarget:       make(map[*LlamaCppTarget]*list.Element),
		targetByConfiguration: make(map[string]*LlamaCppTarget),
		Targets:               list.New(),
	}

	return &LoadBalancer{
		HttpClient:                   httpClient,
		LoadBalancerTargetCollection: loadBalancerTargetCollection,
		Logger:                       logger,
		ServerEventsChannel:          serverEventsChannel,
		StatsdReporter:               statsdReporter,
	}
}
