package loadbalancer

import (
	"container/list"
	"time"

	"github.com/distantmagic/paddler/llamacpp"
)

type LoadBalancerTargetCollection struct {
	LlamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate
	Targets                       *list.List

	elementByTarget       map[*LlamaCppTarget]*list.Element
	targetByConfiguration map[string]*LlamaCppTarget
}

func (self *LoadBalancerTargetCollection) FixTargetOrder(target *LlamaCppTarget) {
	element, ok := self.elementByTarget[target]

	if !ok {
		return
	}

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
	target, ok := self.targetByConfiguration[targetConfiguration.String()]

	if ok {
		return target
	}

	return nil
}

func (self *LoadBalancerTargetCollection) GetHeadTarget() *LlamaCppPickedTarget {
	headElement := self.Targets.Front()

	if headElement == nil {
		return nil
	}

	headTarget := headElement.Value.(*LlamaCppTarget)

	return &LlamaCppPickedTarget{
		Element:        headElement,
		LlamaCppTarget: headTarget,
	}
}

func (self *LoadBalancerTargetCollection) Len() int {
	return self.Targets.Len()
}

func (self *LoadBalancerTargetCollection) RegisterTarget(llamaCppTarget *LlamaCppTarget) {
	self.targetByConfiguration[llamaCppTarget.LlamaCppTargetConfiguration.String()] = llamaCppTarget
	self.LlamaCppHealthStatusAggregate.AddSlotsFrom(llamaCppTarget.LlamaCppHealthStatus)

	if self.Targets.Len() < 1 {
		self.elementByTarget[llamaCppTarget] = self.Targets.PushFront(llamaCppTarget)

		return
	}

	for element := self.Targets.Front(); element != nil; element = element.Next() {
		if element.Value.(*LlamaCppTarget).HasLessSlotsThan(llamaCppTarget) {
			self.elementByTarget[llamaCppTarget] = self.Targets.InsertBefore(llamaCppTarget, element)

			return
		}
	}

	self.elementByTarget[llamaCppTarget] = self.Targets.PushBack(llamaCppTarget)
}

func (self *LoadBalancerTargetCollection) RemoveTarget(llamaCppTarget *LlamaCppTarget) {
	self.LlamaCppHealthStatusAggregate.RemoveSlotsFrom(llamaCppTarget.LlamaCppHealthStatus)
	element := self.elementByTarget[llamaCppTarget]

	if element != nil {
		self.Targets.Remove(element)
	}

	delete(self.targetByConfiguration, llamaCppTarget.LlamaCppTargetConfiguration.String())
}

func (self *LoadBalancerTargetCollection) UpdateTargetWithLlamaCppHealthStatus(
	llamaCppTarget *LlamaCppTarget,
	llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus,
) {
	self.LlamaCppHealthStatusAggregate.IncreaseBy(
		llamaCppTarget.LlamaCppHealthStatus.SlotsIdle-llamaCppHealthStatus.SlotsIdle,
		llamaCppTarget.LlamaCppHealthStatus.SlotsProcessing-llamaCppHealthStatus.SlotsProcessing,
	)

	llamaCppTarget.LlamaCppHealthStatus.CopyFrom(llamaCppHealthStatus)

	llamaCppTarget.LastUpdate = time.Now()
	llamaCppTarget.RemainingTicksUntilRemoved = 3
	llamaCppTarget.TotalUpdates += 1

	self.FixTargetOrder(llamaCppTarget)
}

func (self *LoadBalancerTargetCollection) UseSlot(llamaCppTarget *LlamaCppTarget) {
	targetElement := self.elementByTarget[llamaCppTarget]
	nextTarget := targetElement.Next()

	llamaCppTarget.LlamaCppHealthStatus.SlotsIdle -= 1
	self.LlamaCppHealthStatusAggregate.UseSlot()

	if nextTarget != nil && llamaCppTarget.HasLessSlotsThan(nextTarget.Value.(*LlamaCppTarget)) {
		self.Targets.MoveAfter(targetElement, nextTarget)
	}
}
