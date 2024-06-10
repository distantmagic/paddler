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

	loadBalancer := NewLoadBalancer(
		http.DefaultClient,
		hclog.NewNullLogger(),
		serverEventsChannel,
		&StatsdReporterVoid{},
	)

	assert.Equal(t, 0, loadBalancer.GetStatus().RegisteredTargets)

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

	go loadBalancer.RegisterOrUpdateTarget(
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

	assert.Equal(t, 1, loadBalancer.GetStatus().RegisteredTargets)
	assert.Same(
		t,
		target1,
		loadBalancer.LoadBalancerTargetCollection.GetHeadTarget().LlamaCppTarget.LlamaCppTargetConfiguration,
	)

	go loadBalancer.RegisterOrUpdateTarget(
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

	assert.Equal(t, 2, loadBalancer.GetStatus().RegisteredTargets)
	assert.Same(
		t,
		target1,
		loadBalancer.LoadBalancerTargetCollection.GetHeadTarget().LlamaCppTarget.LlamaCppTargetConfiguration,
	)

	go loadBalancer.RegisterOrUpdateTarget(
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

	assert.Equal(t, 2, loadBalancer.GetStatus().RegisteredTargets)
	assert.Same(
		t,
		target2,
		loadBalancer.LoadBalancerTargetCollection.GetHeadTarget().LlamaCppTarget.LlamaCppTargetConfiguration,
	)
}
