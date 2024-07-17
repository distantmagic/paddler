package loadbalancer

import (
	"container/list"
	"time"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/puzpuzpuz/xsync/v3"
)

type LoadBalancerTargetCollection struct {
	LlamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate
	Targets                       *list.List
	TargetsRBMutex                xsync.RBMutex

	elementByTarget       *xsync.MapOf[*LlamaCppTarget, *list.Element]
	targetByConfiguration *xsync.MapOf[string, *LlamaCppTarget]
}

func (self *LoadBalancerTargetCollection) FixTargetOrder(target *LlamaCppTarget) {
	element := self.getElementByTarget(target)

	if element == nil {
		return
	}

	self.TargetsRBMutex.Lock()
	defer self.TargetsRBMutex.Unlock()

	nextElement := element.Next()

	for nextElement != nil {
		if target.HasLessSlotsThan(nextElement.Value.(*LlamaCppTarget)) {
			self.Targets.MoveAfter(element, nextElement)

			break
		}

		nextElement = nextElement.Next()
	}

	prevElement := element.Prev()

	for prevElement != nil {
		if prevElement.Value.(*LlamaCppTarget).HasLessSlotsThan(target) {
			self.Targets.MoveBefore(element, prevElement)

			break
		}

		prevElement = prevElement.Prev()
	}
}

func (self *LoadBalancerTargetCollection) GetTargetByConfiguration(
	targetConfiguration *LlamaCppTargetConfiguration,
) *LlamaCppTarget {
	target, ok := self.targetByConfiguration.Load(targetConfiguration.Id)

	if ok {
		return target
	}

	return nil
}

func (self *LoadBalancerTargetCollection) GetHeadTarget() *LlamaCppTarget {
	mutexToken := self.TargetsRBMutex.RLock()
	defer self.TargetsRBMutex.RUnlock(mutexToken)

	headElement := self.Targets.Front()

	if headElement == nil {
		return nil
	}

	if headElement.Value == nil {
		return nil
	}

	return headElement.Value.(*LlamaCppTarget)
}

func (self *LoadBalancerTargetCollection) Len() int {
	mutexToken := self.TargetsRBMutex.RLock()
	defer self.TargetsRBMutex.RUnlock(mutexToken)

	return self.Targets.Len()
}

func (self *LoadBalancerTargetCollection) RegisterTarget(llamaCppTarget *LlamaCppTarget) {
	self.setTargetByConfiguration(llamaCppTarget)
	self.LlamaCppHealthStatusAggregate.AddSlotsFrom(llamaCppTarget)

	self.TargetsRBMutex.Lock()
	defer self.TargetsRBMutex.Unlock()

	if self.Targets.Len() < 1 {
		self.elementByTarget.Store(llamaCppTarget, self.Targets.PushFront(llamaCppTarget))

		return
	}

	for element := self.Targets.Front(); element != nil; element = element.Next() {
		if element.Value.(*LlamaCppTarget).HasLessSlotsThan(llamaCppTarget) {
			self.elementByTarget.Store(llamaCppTarget, self.Targets.InsertBefore(llamaCppTarget, element))

			return
		}
	}

	self.elementByTarget.Store(llamaCppTarget, self.Targets.PushBack(llamaCppTarget))
}

func (self *LoadBalancerTargetCollection) RemoveTarget(llamaCppTarget *LlamaCppTarget) {
	self.TargetsRBMutex.Lock()
	defer self.TargetsRBMutex.Unlock()

	self.LlamaCppHealthStatusAggregate.RemoveSlotsFrom(llamaCppTarget)
	element := self.getElementByTarget(llamaCppTarget)

	if element != nil {
		self.Targets.Remove(element)
	}

	self.elementByTarget.Delete(llamaCppTarget)
	self.targetByConfiguration.Delete(llamaCppTarget.LlamaCppTargetConfiguration.Id)
}

func (self *LoadBalancerTargetCollection) UpdateTargetWithLlamaCppHealthStatus(
	llamaCppTarget *LlamaCppTarget,
	llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus,
) {
	slotsIdleDiff, slotsProcessingDiff := llamaCppTarget.SetTickStatus(time.Now(), llamaCppHealthStatus, 3)

	self.LlamaCppHealthStatusAggregate.IncreaseBy(slotsIdleDiff, slotsProcessingDiff)
	self.FixTargetOrder(llamaCppTarget)
}

func (self *LoadBalancerTargetCollection) UseSlot(llamaCppTarget *LlamaCppTarget) {
	targetElement := self.getElementByTarget(llamaCppTarget)

	if targetElement == nil {
		return
	}

	self.TargetsRBMutex.Lock()
	defer self.TargetsRBMutex.Unlock()

	nextTarget := targetElement.Next()

	llamaCppTarget.DecrementIdleSlots()
	self.LlamaCppHealthStatusAggregate.UseSlot()

	if nextTarget != nil && llamaCppTarget.HasLessSlotsThan(nextTarget.Value.(*LlamaCppTarget)) {
		self.Targets.MoveAfter(targetElement, nextTarget)
	}
}

func (self *LoadBalancerTargetCollection) getElementByTarget(target *LlamaCppTarget) *list.Element {
	element, ok := self.elementByTarget.Load(target)

	if !ok {
		return nil
	}

	return element
}

func (self *LoadBalancerTargetCollection) setTargetByConfiguration(target *LlamaCppTarget) {
	targetMutexToken := target.RBMutex.RLock()
	defer target.RBMutex.RUnlock(targetMutexToken)

	self.targetByConfiguration.Store(target.LlamaCppTargetConfiguration.Id, target)
}
