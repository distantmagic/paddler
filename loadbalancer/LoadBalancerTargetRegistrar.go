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
	llamaCppSlotsAggregatedStatus *llamacpp.LlamaCppSlotsAggregatedStatus,
) {
	existingTarget := self.LoadBalancerTargetCollection.GetTargetByConfiguration(targetConfiguration)

	if existingTarget == nil {
		self.registerTarget(
			targetConfiguration,
			llamaCppSlotsAggregatedStatus,
		)
	} else {
		self.updateTarget(
			llamaCppSlotsAggregatedStatus,
			existingTarget,
		)
	}
}

func (self *LoadBalancerTargetRegistrar) registerTarget(
	targetConfiguration *LlamaCppTargetConfiguration,
	llamaCppSlotsAggregatedStatus *llamacpp.LlamaCppSlotsAggregatedStatus,
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
		LlamaCppSlotsAggregatedStatus: llamaCppSlotsAggregatedStatus,
		LlamaCppTargetConfiguration:   targetConfiguration,
		RemainingTicksUntilRemoved:    3,
		ReverseProxy:                  reverseProxy,
	})
}

func (self *LoadBalancerTargetRegistrar) updateTarget(
	llamaCppSlotsAggregatedStatus *llamacpp.LlamaCppSlotsAggregatedStatus,
	existingTarget *LlamaCppTarget,
) {
	self.LoadBalancerTargetCollection.UpdateTargetWithLlamaCppSlotsAggregatedStatus(
		existingTarget,
		llamaCppSlotsAggregatedStatus,
	)
}
