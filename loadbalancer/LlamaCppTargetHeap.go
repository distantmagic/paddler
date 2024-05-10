package loadbalancer

type LlamaCppTargetHeap []*LlamaCppTarget

func (self LlamaCppTargetHeap) Len() int {
	return len(self)
}

func (self LlamaCppTargetHeap) Less(i, j int) bool {
	// inverse comparison, because we want the target with the most idle slots
	// to be at the top
	return !self[i].Less(self[j])
}

func (self LlamaCppTargetHeap) Swap(i, j int) {
	self[i], self[j] = self[j], self[i]
}

func (self *LlamaCppTargetHeap) Push(x any) {
	*self = append(*self, x.(*LlamaCppTarget))
}

func (self *LlamaCppTargetHeap) Pop() any {
	old := *self
	n := len(old)
	x := old[n-1]
	*self = old[0 : n-1]

	return x
}
