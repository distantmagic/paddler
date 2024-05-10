package loadbalancer

import (
	"container/heap"
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
	HttpClient *http.Client
	Logger     hclog.Logger
	targets    *LlamaCppTargetHeap
}

func (self *LoadBalancer) Balance(request *http.Request) (*url.URL, error) {
	if self.targets.Len() < 1 {
		return nil, ErrorNoTargetsAvailable
	}

	headTarget := (*self.targets)[0]

	targetUrl := headTarget.
		LlamaCppClient.
		LlamaCppConfiguration.
		HttpAddress.
		BuildUrlWithPath("")

	headTarget.LlamaCppHealthStatus.SlotsIdle -= 1
	heap.Fix(self.targets, 0)

	self.Logger.Debug(
		"balancing",
		"target", targetUrl,
		"slots", headTarget.LlamaCppHealthStatus.SlotsIdle,
	)

	return targetUrl, nil
}

func (self *LoadBalancer) GetStatus() *LoadBalancerStatus {
	return &LoadBalancerStatus{
		RegisteredTargets: self.targets.Len(),
	}
}

func (self *LoadBalancer) RegisterTarget(
	serverEventsChannel chan goroutine.ResultMessage,
	targetConfiguration *LlamaCppTargetConfiguration,
) {
	self.Logger.Debug(
		"registering target",
		"host", targetConfiguration.LlamaCppConfiguration.HttpAddress.GetHostWithPort(),
	)

	llamaCppClient := &llamacpp.LlamaCppClient{
		HttpClient:            self.HttpClient,
		LlamaCppConfiguration: targetConfiguration.LlamaCppConfiguration,
	}

	responseChannel := make(chan llamacpp.LlamaCppHealthStatus)

	go llamaCppClient.GetHealth(responseChannel)

	healthStatus := <-responseChannel

	if healthStatus.Error != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to register target",
			Error:   healthStatus.Error,
		}

		return
	}

	llamaCppTarget := &LlamaCppTarget{
		LlamaCppClient:       llamaCppClient,
		LlamaCppHealthStatus: &healthStatus,
	}

	heap.Push(self.targets, llamaCppTarget)

	serverEventsChannel <- goroutine.ResultMessage{
		Comment: "registered target",
	}
}
