package loadbalancer

import (
	"net/http"
	"net/http/httputil"
	"time"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/hashicorp/go-hclog"
)

type LoadBalancerTargetRegistrar struct {
	HttpClient                   *http.Client
	LoadBalancerTargetCollection *LoadBalancerTargetCollection
	Logger                       hclog.Logger
}

func (self *LoadBalancerTargetRegistrar) RegisterOrUpdateTarget(
	targetConfiguration *LlamaCppTargetConfiguration,
	llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus,
) {
	existingTarget := self.LoadBalancerTargetCollection.GetTargetByConfiguration(targetConfiguration)

	if existingTarget == nil {
		self.registerTarget(
			targetConfiguration,
			llamaCppHealthStatus,
		)
	} else {
		self.updateTarget(
			llamaCppHealthStatus,
			existingTarget,
		)
	}
}

func (self *LoadBalancerTargetRegistrar) registerTarget(
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
}

func (self *LoadBalancerTargetRegistrar) updateTarget(
	llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus,
	existingTarget *LlamaCppTarget,
) {
	self.LoadBalancerTargetCollection.UpdateTargetWithLlamaCppHealthStatus(
		existingTarget,
		llamaCppHealthStatus,
	)
}
