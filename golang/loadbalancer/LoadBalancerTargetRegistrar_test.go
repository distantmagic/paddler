package loadbalancer

import (
	"net/http"
	"testing"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/hashicorp/go-hclog"
	"github.com/stretchr/testify/assert"
)

func TestTargetOrderIsPreserved(t *testing.T) {
	llamaCppHealthStatusAggregate := &LlamaCppHealthStatusAggregate{
		AggregatedHealthStatus: &llamacpp.LlamaCppSlotsAggregatedStatus{
			Status: llamacpp.Ok,
		},
	}

	loadBalancerTargetRegistrar := &LoadBalancerTargetRegistrar{
		HttpClient:                   http.DefaultClient,
		LoadBalancerConfiguration:    &LoadBalancerConfiguration{},
		LoadBalancerTargetCollection: NewLoadBalancerTargetCollection(llamaCppHealthStatusAggregate),
		Logger:                       hclog.NewNullLogger(),
	}

	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.Len())

	target1 := &LlamaCppTargetConfiguration{
		Id: "target1",
		LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{
				Host:   "127.0.0.1",
				Port:   8081,
				Scheme: "http",
			},
		},
	}

	loadBalancerTargetRegistrar.RegisterOrUpdateTarget(
		target1,
		&llamacpp.LlamaCppSlotsAggregatedStatus{
			SlotsIdle:       10,
			SlotsProcessing: 0,
			Status:          llamacpp.Ok,
			Error:           nil,
		},
	)

	assert.NotNil(t, loadBalancerTargetRegistrar.LoadBalancerTargetCollection)
	assert.Equal(t, 1, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.Len())
	assert.Equal(t, 10, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target1.Id).LlamaCppSlotsAggregatedStatus.SlotsIdle)
	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target1.Id).LlamaCppSlotsAggregatedStatus.SlotsProcessing)
	assert.Equal(t, 10, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsIdle)
	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsProcessing)

	headTarget := loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetHeadTarget()

	assert.NotNil(t, headTarget)
	assert.Same(t, target1, headTarget.LlamaCppTargetConfiguration)

	target2 := &LlamaCppTargetConfiguration{
		Id: "target2",
		LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{
				Host:   "127.0.0.1",
				Port:   8082,
				Scheme: "http",
			},
		},
	}

	loadBalancerTargetRegistrar.RegisterOrUpdateTarget(
		target2,
		&llamacpp.LlamaCppSlotsAggregatedStatus{
			SlotsIdle:       8,
			SlotsProcessing: 0,
			Status:          llamacpp.Ok,
			Error:           nil,
		},
	)

	assert.Equal(t, 2, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.Len())
	assert.Equal(t, 10, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target1.Id).LlamaCppSlotsAggregatedStatus.SlotsIdle)
	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target1.Id).LlamaCppSlotsAggregatedStatus.SlotsProcessing)
	assert.Equal(t, 8, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target2.Id).LlamaCppSlotsAggregatedStatus.SlotsIdle)
	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target2.Id).LlamaCppSlotsAggregatedStatus.SlotsProcessing)
	assert.Equal(t, 18, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsIdle)
	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsProcessing)
	assert.Same(
		t,
		target1,
		loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetHeadTarget().LlamaCppTargetConfiguration,
	)

	loadBalancerTargetRegistrar.RegisterOrUpdateTarget(
		target2,
		&llamacpp.LlamaCppSlotsAggregatedStatus{
			SlotsIdle:       11,
			SlotsProcessing: 0,
			Status:          llamacpp.Ok,
			Error:           nil,
		},
	)

	assert.Equal(t, 2, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.Len())
	assert.Equal(t, 10, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target1.Id).LlamaCppSlotsAggregatedStatus.SlotsIdle)
	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target1.Id).LlamaCppSlotsAggregatedStatus.SlotsProcessing)
	assert.Equal(t, 11, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target2.Id).LlamaCppSlotsAggregatedStatus.SlotsIdle)
	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetTargetById(target2.Id).LlamaCppSlotsAggregatedStatus.SlotsProcessing)
	assert.Equal(t, 21, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsIdle)
	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.LlamaCppHealthStatusAggregate.AggregatedHealthStatus.SlotsProcessing)
	assert.Same(
		t,
		target2,
		loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetHeadTarget().LlamaCppTargetConfiguration,
	)
}
