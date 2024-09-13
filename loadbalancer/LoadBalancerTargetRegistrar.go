package loadbalancer

import (
	"net/http"
	"time"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/hashicorp/go-hclog"
)

type LoadBalancerTargetRegistrar struct {
	HttpClient                   *http.Client
	LoadBalancerConfiguration    *LoadBalancerConfiguration
	LoadBalancerTargetCollection *LoadBalancerTargetCollection
	Logger                       hclog.Logger
}

func (self *LoadBalancerTargetRegistrar) RegisterOrUpdateTarget(
	targetConfiguration *LlamaCppTargetConfiguration,
	llamaCppSlotsAggregatedStatus *llamacpp.LlamaCppSlotsAggregatedStatus,
) {
	existingTarget := self.LoadBalancerTargetCollection.GetTargetById(targetConfiguration.Id)

	if existingTarget == nil {
		self.registerTarget(
			targetConfiguration,
			llamaCppSlotsAggregatedStatus,
		)
	} else {
		self.LoadBalancerTargetCollection.UpdateTargetWithLlamaCppSlotsAggregatedStatus(
			existingTarget,
			llamaCppSlotsAggregatedStatus,
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

	self.LoadBalancerTargetCollection.RegisterTarget(&LlamaCppTarget{
		LastUpdate: time.Now(),
		LlamaCppClient: &llamacpp.LlamaCppClient{
			HttpClient:            self.HttpClient,
			LlamaCppConfiguration: targetConfiguration.LlamaCppConfiguration,
		},
		LlamaCppSlotsAggregatedStatus: llamaCppSlotsAggregatedStatus,
		LlamaCppTargetConfiguration:   targetConfiguration,
		RemainingTicksUntilRemoved:    3,
		ReverseProxy: CreateLlamaCppTargetReverseProxy(
			self.Logger.Named("ReverseProxy"),
			self.LoadBalancerConfiguration,
			targetConfiguration,
		),
	})
}
