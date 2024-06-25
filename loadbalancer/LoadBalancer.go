package loadbalancer

import (
	"context"
	"errors"

	"github.com/hashicorp/go-hclog"
)

var (
	ErrorNoTargetsAvailable = errors.New("no targets available")
)

type LoadBalancer struct {
	LoadBalancerTargetCollection *LoadBalancerTargetCollection
	Logger                       hclog.Logger
	RequestBuffer                *RequestBuffer
}

func (self *LoadBalancer) Balance(
	ctx context.Context,
	balancingAttemptStatusChannel chan<- *BalancingAttemptStatus,
	request *LoadBalancerRequest,
) {
	headPickedTarget := self.LoadBalancerTargetCollection.GetHeadTarget()

	if headPickedTarget == nil {
		balancingAttemptStatusChannel <- &BalancingAttemptStatus{
			Error: ErrorNoTargetsAvailable,
		}

		return
	}

	headTarget := headPickedTarget.LlamaCppTarget

	if request.IsSlottable() {
		self.LoadBalancerTargetCollection.UseSlot(headTarget)
	}

	targetUrl := headTarget.
		LlamaCppClient.
		LlamaCppConfiguration.
		HttpAddress.
		BuildUrlWithPath("")

	self.Logger.Debug(
		"balancing",
		"target", targetUrl,
		"slots", headTarget.LlamaCppHealthStatus.SlotsIdle,
	)

	balancingAttemptStatusChannel <- &BalancingAttemptStatus{
		TargetUrl: targetUrl,
	}
}

func (self *LoadBalancer) GetStatus() *LoadBalancerStatus {
	return &LoadBalancerStatus{
		RegisteredTargets: self.LoadBalancerTargetCollection.Len(),
	}
}
