package loadbalancer

import (
	"container/heap"
	"errors"
	"net/http"
	"net/url"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/hashicorp/go-hclog"
)

var (
	ErrorNoTargetsAvailable = errors.New("no targets available")
)

type LoadBalancer struct {
	Logger  hclog.Logger
	targets *LlamaCppTargetHeap
}

func (self *LoadBalancer) Balance(request *http.Request) (*url.URL, error) {
	headTarget := (*self.targets)[0]

	if headTarget == nil {
		return nil, ErrorNoTargetsAvailable
	}

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

func (self *LoadBalancer) RegisterTarget(targetConfiguration *LlamaCppTargetConfiguration) {
	heap.Push(self.targets, &LlamaCppTarget{
		LlamaCppClient: &llamacpp.LlamaCppClient{
			HttpClient:            http.DefaultClient,
			LlamaCppConfiguration: targetConfiguration.LlamaCppConfiguration,
		},
		LlamaCppHealthStatus: &llamacpp.LlamaCppHealthStatus{
			SlotsIdle: 10,
		},
	})
}
