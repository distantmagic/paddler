package loadbalancer

import (
	"container/heap"
	"sync"
)

type LoadBalancerTargetCollection struct {
	mutex      sync.Mutex
	targetMap  map[string]*LlamaCppTarget
	targetHeap *LlamaCppTargetHeap
}

func (self *LoadBalancerTargetCollection) HasTargetConfiguration(
	targetConfiguration *LlamaCppTargetConfiguration,
) bool {
	_, ok := self.targetMap[targetConfiguration.String()]

	return ok
}

func (self *LoadBalancerTargetCollection) GetHead() *LlamaCppTarget {
	if self.targetHeap.Len() < 1 {
		return nil
	}

	return (*self.targetHeap)[0]
}

func (self *LoadBalancerTargetCollection) GetForBalancingSlot() *LlamaCppTarget {
	headTarget := self.GetHead()

	if headTarget == nil {
		return nil
	}

	self.mutex.Lock()
	defer self.mutex.Unlock()

	headTarget.LlamaCppHealthStatus.SlotsIdle -= 1
	heap.Fix(self.targetHeap, 0)

	return headTarget
}

func (self *LoadBalancerTargetCollection) Len() int {
	return self.targetHeap.Len()
}

func (self *LoadBalancerTargetCollection) RegisterTarget(llamaCppTarget *LlamaCppTarget) {
	self.targetMap[llamaCppTarget.LlamaCppTargetConfiguration.String()] = llamaCppTarget
	heap.Push(self.targetHeap, llamaCppTarget)
}
