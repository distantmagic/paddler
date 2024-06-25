package loadbalancer

import (
	"net/http"
	"testing"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/hashicorp/go-hclog"
	"github.com/stretchr/testify/assert"
)

func TestTargetOrderIsPreserved(t *testing.T) {
	serverEventsChannel := make(chan goroutine.ResultMessage)

	llamaCppHealthStatusAggregate := &LlamaCppHealthStatusAggregate{
		AggregatedHealthStatus: &llamacpp.LlamaCppHealthStatus{
			Status: llamacpp.Ok,
		},
	}

	loadBalancerTargetRegistrar := &LoadBalancerTargetRegistrar{
		HttpClient:                   http.DefaultClient,
		LoadBalancerTargetCollection: NewLoadBalancerTargetCollection(llamaCppHealthStatusAggregate),
		Logger:                       hclog.NewNullLogger(),
	}

	assert.Equal(t, 0, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.Len())

	target1 := &LlamaCppTargetConfiguration{
		LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{
				Host:   "127.0.0.1",
				Port:   8081,
				Scheme: "http",
			},
		},
	}

	target2 := &LlamaCppTargetConfiguration{
		LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{
				Host:   "127.0.0.1",
				Port:   8082,
				Scheme: "http",
			},
		},
	}

	go loadBalancerTargetRegistrar.RegisterOrUpdateTarget(
		serverEventsChannel,
		target1,
		&llamacpp.LlamaCppHealthStatus{
			Status:          llamacpp.Ok,
			SlotsIdle:       10,
			SlotsProcessing: 0,
			Error:           nil,
		},
	)

	<-serverEventsChannel

	assert.Equal(t, 1, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.Len())
	assert.Same(
		t,
		target1,
		loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetHeadTarget().LlamaCppTarget.LlamaCppTargetConfiguration,
	)

	go loadBalancerTargetRegistrar.RegisterOrUpdateTarget(
		serverEventsChannel,
		target2,
		&llamacpp.LlamaCppHealthStatus{
			Status:          llamacpp.Ok,
			SlotsIdle:       8,
			SlotsProcessing: 0,
			Error:           nil,
		},
	)

	<-serverEventsChannel

	assert.Equal(t, 2, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.Len())
	assert.Same(
		t,
		target1,
		loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetHeadTarget().LlamaCppTarget.LlamaCppTargetConfiguration,
	)

	go loadBalancerTargetRegistrar.RegisterOrUpdateTarget(
		serverEventsChannel,
		target2,
		&llamacpp.LlamaCppHealthStatus{
			Status:          llamacpp.Ok,
			SlotsIdle:       11,
			SlotsProcessing: 0,
			Error:           nil,
		},
	)

	<-serverEventsChannel

	assert.Equal(t, 2, loadBalancerTargetRegistrar.LoadBalancerTargetCollection.Len())
	assert.Same(
		t,
		target2,
		loadBalancerTargetRegistrar.LoadBalancerTargetCollection.GetHeadTarget().LlamaCppTarget.LlamaCppTargetConfiguration,
	)
}
