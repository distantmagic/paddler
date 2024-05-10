package loadbalancer

import (
	"errors"
	"net/http"
	"net/url"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/emirpasic/gods/v2/trees/binaryheap"
	"github.com/hashicorp/go-hclog"
)

type LoadBalancer struct {
	Logger  hclog.Logger
	targets *binaryheap.Heap[*LlamaCppTarget]
}

func (self *LoadBalancer) Balance(request *http.Request) (*url.URL, error) {
	headTarget, ok := self.targets.Peek()

	if !ok || headTarget == nil {
		return nil, errors.New("no targets available")
	}

	targetUrl := headTarget.
		LlamaCppClient.
		LlamaCppConfiguration.
		HttpAddress.
		BuildUrlWithPath("")

	headTarget.LlamaCppHealthStatus.SlotsIdle -= 1

	self.Logger.Debug("balancing", "target", targetUrl)

	return targetUrl, nil
}

func (self *LoadBalancer) RegisterTarget(targetConfiguration *LlamaCppTargetConfiguration) {
	self.targets.Push(&LlamaCppTarget{
		LlamaCppClient: &llamacpp.LlamaCppClient{
			HttpClient:            http.DefaultClient,
			LlamaCppConfiguration: targetConfiguration.LlamaCppConfiguration,
		},
		LlamaCppHealthStatus: &llamacpp.LlamaCppHealthStatus{
			SlotsIdle: 10,
		},
	})
}
