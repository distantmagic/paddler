package loadbalancer

import (
	"slices"
	"time"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/puzpuzpuz/xsync/v3"
)

type LoadBalancerTargetCollection struct {
	LlamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate `json:"-"`				
	Targets                       []*LlamaCppTarget				 `json:"targets"`

	targetById *xsync.MapOf[string, *LlamaCppTarget]			 `json:"-"`
	RBMutex    xsync.RBMutex									 `json:"-"`
}

func (self *LoadBalancerTargetCollection) GetTargetById(targetId string) *LlamaCppTarget {
	target, ok := self.targetById.Load(targetId)

	if ok {
		return target
	}

	return nil
}

func (self *LoadBalancerTargetCollection) GetHeadTarget() *LlamaCppTarget {
	mutexToken := self.RBMutex.RLock()
	defer self.RBMutex.RUnlock(mutexToken)

	if len(self.Targets) > 0 {
		return self.Targets[0]
	}

	return nil
}

func (self *LoadBalancerTargetCollection) Len() int {
	mutexToken := self.RBMutex.RLock()
	defer self.RBMutex.RUnlock(mutexToken)

	return len(self.Targets)
}

func (self *LoadBalancerTargetCollection) RegisterTarget(llamaCppTarget *LlamaCppTarget) {
	self.RBMutex.Lock()
	defer self.RBMutex.Unlock()

	self.Targets = append(self.Targets, llamaCppTarget)
	self.sort()

	self.targetById.Store(llamaCppTarget.LlamaCppTargetConfiguration.Id, llamaCppTarget)
	self.LlamaCppHealthStatusAggregate.AddSlotsFrom(llamaCppTarget)
}

func (self *LoadBalancerTargetCollection) RemoveTarget(llamaCppTarget *LlamaCppTarget) {
	self.RBMutex.Lock()
	defer self.RBMutex.Unlock()

	for i, target := range self.Targets {
		if target == llamaCppTarget {
			self.Targets = append(self.Targets[:i], self.Targets[i+1:]...)

			break
		}
	}

	self.targetById.Delete(llamaCppTarget.LlamaCppTargetConfiguration.Id)
	self.LlamaCppHealthStatusAggregate.RemoveSlotsFrom(llamaCppTarget)
}

func (self *LoadBalancerTargetCollection) UpdateTargetWithLlamaCppSlotsAggregatedStatus(
	llamaCppTarget *LlamaCppTarget,
	llamaCppSlotsAggregatedStatus *llamacpp.LlamaCppSlotsAggregatedStatus,
) {
	self.RBMutex.Lock()
	defer self.RBMutex.Unlock()

	slotsIdleDiff, slotsProcessingDiff := llamaCppTarget.SetTickStatus(time.Now(), llamaCppSlotsAggregatedStatus, 3)
	self.LlamaCppHealthStatusAggregate.IncreaseBy(slotsIdleDiff, slotsProcessingDiff)
	self.sort()
}

func (self *LoadBalancerTargetCollection) UseSlot(llamaCppTarget *LlamaCppTarget) {
	self.RBMutex.Lock()
	defer self.RBMutex.Unlock()

	self.LlamaCppHealthStatusAggregate.UseSlot()
	llamaCppTarget.DecrementIdleSlots()
	self.sort()
}

func (self *LoadBalancerTargetCollection) sort() {
	slices.SortFunc(self.Targets, LlamaCppTargetBalancingCompare)
}
