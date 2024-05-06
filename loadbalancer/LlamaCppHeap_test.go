package loadbalancer

import (
	"container/heap"
	"testing"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/stretchr/testify/assert"
)

func TestHeapIsMaintained(t *testing.T) {
	llamaHeap := &LlamaCppHeap{}

	heap.Init(llamaHeap)

	status1 := llamacpp.LlamaCppHealthStatus{SlotsIdle: 10}
	status2 := llamacpp.LlamaCppHealthStatus{SlotsIdle: 20}
	status3 := llamacpp.LlamaCppHealthStatus{SlotsIdle: 15}

	heap.Push(llamaHeap, &LlamaCppTarget{
		LlamaCppHealthStatus: &status1,
	})
	heap.Push(llamaHeap, &LlamaCppTarget{
		LlamaCppHealthStatus: &status2,
	})
	heap.Push(llamaHeap, &LlamaCppTarget{
		LlamaCppHealthStatus: &status3,
	})

	assert.Equal(t, 3, llamaHeap.Len())
	assert.Same(t, &status2, llamaHeap.Head().LlamaCppHealthStatus)

	popped := heap.Pop(llamaHeap).(*LlamaCppTarget)
	assert.Same(t, &status2, popped.LlamaCppHealthStatus)
	assert.Equal(t, 2, llamaHeap.Len())

	popped = heap.Pop(llamaHeap).(*LlamaCppTarget)
	assert.Same(t, &status3, popped.LlamaCppHealthStatus)
	assert.Equal(t, 1, llamaHeap.Len())

	popped = heap.Pop(llamaHeap).(*LlamaCppTarget)
	assert.Same(t, &status1, popped.LlamaCppHealthStatus)
	assert.Equal(t, 0, llamaHeap.Len())
}

func TestHeapIsFixed(t *testing.T) {
	llamaHeap := &LlamaCppHeap{}

	heap.Init(llamaHeap)

	status1 := llamacpp.LlamaCppHealthStatus{SlotsIdle: 10}
	status2 := llamacpp.LlamaCppHealthStatus{SlotsIdle: 20}
	status3 := llamacpp.LlamaCppHealthStatus{SlotsIdle: 15}

	heap.Push(llamaHeap, &LlamaCppTarget{
		LlamaCppHealthStatus: &status1,
	})
	heap.Push(llamaHeap, &LlamaCppTarget{
		LlamaCppHealthStatus: &status2,
	})
	heap.Push(llamaHeap, &LlamaCppTarget{
		LlamaCppHealthStatus: &status3,
	})

	status2.SlotsIdle = 5

	heap.Fix(llamaHeap, 0)

	assert.Equal(t, 3, llamaHeap.Len())
	assert.Same(t, &status3, llamaHeap.Head().LlamaCppHealthStatus)

	popped := heap.Pop(llamaHeap).(*LlamaCppTarget)
	assert.Same(t, &status3, popped.LlamaCppHealthStatus)
	assert.Equal(t, 2, llamaHeap.Len())

	popped = heap.Pop(llamaHeap).(*LlamaCppTarget)
	assert.Same(t, &status1, popped.LlamaCppHealthStatus)
	assert.Equal(t, 1, llamaHeap.Len())

	popped = heap.Pop(llamaHeap).(*LlamaCppTarget)
	assert.Same(t, &status2, popped.LlamaCppHealthStatus)
	assert.Equal(t, 0, llamaHeap.Len())
}
