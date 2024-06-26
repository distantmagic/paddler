package loadbalancer

import (
	"errors"

	"github.com/hashicorp/go-hclog"
)

var (
	ErrorNoSlotsAvailable   = errors.New("no slots available")
	ErrorNoTargetsAvailable = errors.New("no targets available")
)

type LoadBalancer struct {
	LoadBalancerTargetCollection *LoadBalancerTargetCollection
	Logger                       hclog.Logger
}

func (self *LoadBalancer) Balance(
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

	if headTarget.LlamaCppHealthStatus.SlotsIdle < 1 {
		balancingAttemptStatusChannel <- &BalancingAttemptStatus{
			Error: ErrorNoSlotsAvailable,
		}

		return
	}

	self.LoadBalancerTargetCollection.UseSlot(headTarget)
	balancingAttemptStatusChannel <- &BalancingAttemptStatus{
		LlamaCppTarget: headTarget,
	}
}

func (self *LoadBalancer) GetStatus() *LoadBalancerStatus {
	return &LoadBalancerStatus{
		RegisteredTargets: self.LoadBalancerTargetCollection.Len(),
	}
}
