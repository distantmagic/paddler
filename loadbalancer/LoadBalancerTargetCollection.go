package loadbalancer

import (
	"container/list"
)

type LoadBalancerTargetCollection struct {
	elementByTarget       map[*LlamaCppTarget]*list.Element
	Targets               *list.List
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

func (self *LoadBalancerTargetCollection) GetTargetWithFreeSlotsForBalancing() *LlamaCppPickedTarget {
	pickedTarget := self.GetHeadTarget()

	if pickedTarget == nil || pickedTarget.LlamaCppTarget.LlamaCppHealthStatus.SlotsIdle < 1 {
		return nil
	}

	nextTarget := pickedTarget.Element.Next()

	pickedTarget.LlamaCppTarget.LlamaCppHealthStatus.SlotsIdle -= 1

	if nextTarget != nil && pickedTarget.LlamaCppTarget.HasLessSlotsThan(nextTarget.Value.(*LlamaCppTarget)) {
		self.Targets.MoveAfter(pickedTarget.Element, nextTarget)
	}

	return pickedTarget
}

func (self *LoadBalancerTargetCollection) Len() int {
	return self.Targets.Len()
}

func (self *LoadBalancerTargetCollection) RegisterTarget(llamaCppTarget *LlamaCppTarget) {
	self.targetByConfiguration[llamaCppTarget.LlamaCppTargetConfiguration.String()] = llamaCppTarget

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
	element := self.elementByTarget[llamaCppTarget]

	if element != nil {
		self.Targets.Remove(element)
	}

	delete(self.targetByConfiguration, llamaCppTarget.LlamaCppTargetConfiguration.String())
}
