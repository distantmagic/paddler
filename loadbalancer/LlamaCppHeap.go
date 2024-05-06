package loadbalancer

type LlamaCppHeap [](*LlamaCppTarget)

func (self *LlamaCppHeap) Head() *LlamaCppTarget {
	return (*self)[0]
}

func (self LlamaCppHeap) Len() int {
	return len(self)
}

func (self LlamaCppHeap) Less(i, j int) bool {
	return self[i].LlamaCppHealthStatus.SlotsIdle > self[j].LlamaCppHealthStatus.SlotsIdle
}

func (self *LlamaCppHeap) Pop() interface{} {
	old := *self
	n := len(old)
	x := old[n-1]
	*self = old[0 : n-1]

	return x
}

func (self *LlamaCppHeap) Push(x interface{}) {
	*self = append(*self, x.(*LlamaCppTarget))
}

func (self LlamaCppHeap) Swap(i, j int) {
	self[i], self[j] = self[j], self[i]
}
