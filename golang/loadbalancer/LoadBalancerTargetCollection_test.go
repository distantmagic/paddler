package loadbalancer

import (
	"testing"
	"time"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/stretchr/testify/assert"
)

func mockLlamaCppTarget(id string, port uint) *LlamaCppTarget {
	return &LlamaCppTarget{
		LastUpdate:     time.Now(),
		LlamaCppClient: nil,
		LlamaCppSlotsAggregatedStatus: &llamacpp.LlamaCppSlotsAggregatedStatus{
			SlotsIdle:       4,
			SlotsProcessing: 0,
			Status:          llamacpp.Ok,
		},
		LlamaCppTargetConfiguration: &LlamaCppTargetConfiguration{
			Id:   id + "_id",
			Name: id + "_name",
			LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
				HttpAddress: &netcfg.HttpAddressConfiguration{
					Host:   "127.0.0.1",
					Port:   port,
					Scheme: "http",
				},
			},
		},
		RemainingTicksUntilRemoved: 3,
		ReverseProxy:               nil,
		TotalUpdates:               1,
	}
}

func mockLoadBalancerTargetCollection() *LoadBalancerTargetCollection {
	llamaCppHealthStatusAggregate := &LlamaCppHealthStatusAggregate{
		AggregatedHealthStatus: &llamacpp.LlamaCppSlotsAggregatedStatus{
			Status: llamacpp.Ok,
		},
	}

	return NewLoadBalancerTargetCollection(llamaCppHealthStatusAggregate)
}

func TestTargetSlotsAreUsed(t *testing.T) {
	loadBalancerTargetCollection := mockLoadBalancerTargetCollection()

	target1 := mockLlamaCppTarget("1", 8080)
	target2 := mockLlamaCppTarget("2", 8081)
	target3 := mockLlamaCppTarget("3", 8082)

	loadBalancerTargetCollection.RegisterTarget(target1)
	loadBalancerTargetCollection.RegisterTarget(target2)
	loadBalancerTargetCollection.RegisterTarget(target3)

	assert.Equal(t, 12, loadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsIdle)
	assert.Equal(t, 0, loadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsProcessing)

	assert.Same(t, target1, loadBalancerTargetCollection.GetHeadTarget())

	loadBalancerTargetCollection.UseSlot(target1)

	assert.Equal(t, 3, target1.LlamaCppSlotsAggregatedStatus.SlotsIdle)
	assert.Equal(t, 1, target1.LlamaCppSlotsAggregatedStatus.SlotsProcessing)

	assert.Equal(t, 11, loadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsIdle)
	assert.Equal(t, 1, loadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsProcessing)

	assert.Same(t, target2, loadBalancerTargetCollection.GetHeadTarget())

	loadBalancerTargetCollection.UpdateTargetWithLlamaCppSlotsAggregatedStatus(
		target2,
		&llamacpp.LlamaCppSlotsAggregatedStatus{
			SlotsIdle:       0,
			SlotsProcessing: 0,
			Status:          llamacpp.Ok,
		},
	)

	assert.Equal(t, 3, target1.LlamaCppSlotsAggregatedStatus.SlotsIdle)
	assert.Equal(t, 1, target1.LlamaCppSlotsAggregatedStatus.SlotsProcessing)

	assert.Equal(t, 0, target2.LlamaCppSlotsAggregatedStatus.SlotsIdle)
	assert.Equal(t, 0, target2.LlamaCppSlotsAggregatedStatus.SlotsProcessing)

	assert.Equal(t, 4, target3.LlamaCppSlotsAggregatedStatus.SlotsIdle)
	assert.Equal(t, 0, target3.LlamaCppSlotsAggregatedStatus.SlotsProcessing)

	assert.Equal(t, 7, loadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsIdle)
	assert.Equal(t, 1, loadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsProcessing)

	assert.Same(t, target3, loadBalancerTargetCollection.GetHeadTarget())
}
