package loadbalancer

import (
	"errors"
	"net/http"
	"net/url"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/hashicorp/go-hclog"
)

var (
	ErrorNoTargetsAvailable = errors.New("no targets available")
)

type LoadBalancer struct {
	HttpClient                   *http.Client
	LoadBalancerTargetCollection *LoadBalancerTargetCollection
	Logger                       hclog.Logger
}

func (self *LoadBalancer) Balance(request *LoadBalancerRequest) (*url.URL, error) {
	headTarget := self.GetLlamaCppTargetForRequest(request)

	if headTarget == nil {
		return nil, ErrorNoTargetsAvailable
	}

	targetUrl := headTarget.
		LlamaCppClient.
		LlamaCppConfiguration.
		HttpAddress.
		BuildUrlWithPath("")

	self.Logger.Debug(
		"balancing",
		"target", targetUrl,
		"slots", headTarget.LlamaCppHealthStatus.SlotsIdle,
	)

	return targetUrl, nil
}

func (self *LoadBalancer) GetLlamaCppTargetForRequest(request *LoadBalancerRequest) *LlamaCppTarget {
	if request.IsSlottable() {
		return self.LoadBalancerTargetCollection.GetForBalancingSlot()
	}

	return self.LoadBalancerTargetCollection.GetHead()
}

func (self *LoadBalancer) GetStatus() *LoadBalancerStatus {
	return &LoadBalancerStatus{
		RegisteredTargets: self.LoadBalancerTargetCollection.Len(),
	}
}

func (self *LoadBalancer) RegisterTarget(
	serverEventsChannel chan goroutine.ResultMessage,
	targetConfiguration *LlamaCppTargetConfiguration,
) {
	if self.LoadBalancerTargetCollection.HasTargetConfiguration(targetConfiguration) {
		// nothing to do here
		return
	}

	self.Logger.Debug(
		"registering target",
		"host", targetConfiguration.LlamaCppConfiguration.HttpAddress.GetHostWithPort(),
	)

	llamaCppClient := &llamacpp.LlamaCppClient{
		HttpClient:            self.HttpClient,
		LlamaCppConfiguration: targetConfiguration.LlamaCppConfiguration,
	}

	responseChannel := make(chan llamacpp.LlamaCppHealthStatus)

	defer close(responseChannel)

	go llamaCppClient.GetHealth(responseChannel)

	healthStatus := <-responseChannel

	if healthStatus.Error != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to register target",
			Error:   healthStatus.Error,
		}

		return
	}

	self.LoadBalancerTargetCollection.RegisterTarget(&LlamaCppTarget{
		LlamaCppClient:              llamaCppClient,
		LlamaCppHealthStatus:        &healthStatus,
		LlamaCppTargetConfiguration: targetConfiguration,
	})

	serverEventsChannel <- goroutine.ResultMessage{
		Comment: "registered target",
	}
}
