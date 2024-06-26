package loadbalancer

import (
	"net/http"
	"net/http/httputil"
	"time"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/hashicorp/go-hclog"
)

type LoadBalancerTargetRegistrar struct {
	HttpClient                   *http.Client
	LoadBalancerTargetCollection *LoadBalancerTargetCollection
	Logger                       hclog.Logger
}

func (self *LoadBalancerTargetRegistrar) RegisterOrUpdateTarget(
	serverEventsChannel chan<- goroutine.ResultMessage,
	targetConfiguration *LlamaCppTargetConfiguration,
	llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus,
) {
	existingTarget := self.LoadBalancerTargetCollection.GetTargetByConfiguration(targetConfiguration)

	if existingTarget == nil {
		self.registerTarget(
			serverEventsChannel,
			targetConfiguration,
			llamaCppHealthStatus,
		)
	} else {
		self.updateTarget(
			serverEventsChannel,
			targetConfiguration,
			llamaCppHealthStatus,
			existingTarget,
		)
	}
}

func (self *LoadBalancerTargetRegistrar) registerTarget(
	serverEventsChannel chan<- goroutine.ResultMessage,
	targetConfiguration *LlamaCppTargetConfiguration,
	llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus,
) {
	self.Logger.Debug(
		"registering target",
		"host", targetConfiguration.LlamaCppConfiguration.HttpAddress.GetHostWithPort(),
	)

	reverseProxy := httputil.NewSingleHostReverseProxy(
		targetConfiguration.LlamaCppConfiguration.HttpAddress.GetBaseUrl(),
	)

	reverseProxy.ErrorLog = self.Logger.Named("ReverseProxy").StandardLogger(&hclog.StandardLoggerOptions{
		InferLevels: true,
	})

	self.LoadBalancerTargetCollection.RegisterTarget(&LlamaCppTarget{
		LastUpdate: time.Now(),
		LlamaCppClient: &llamacpp.LlamaCppClient{
			HttpClient:            self.HttpClient,
			LlamaCppConfiguration: targetConfiguration.LlamaCppConfiguration,
		},
		LlamaCppHealthStatus:        llamaCppHealthStatus,
		LlamaCppTargetConfiguration: targetConfiguration,
		RemainingTicksUntilRemoved:  3,
		ReverseProxy:                reverseProxy,
	})

	serverEventsChannel <- goroutine.ResultMessage{
		Comment: "registered target",
	}
}

func (self *LoadBalancerTargetRegistrar) updateTarget(
	serverEventsChannel chan<- goroutine.ResultMessage,
	targetConfiguration *LlamaCppTargetConfiguration,
	llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus,
	existingTarget *LlamaCppTarget,
) {
	self.LoadBalancerTargetCollection.UpdateTargetWithLlamaCppHealthStatus(
		existingTarget,
		llamaCppHealthStatus,
	)
}
