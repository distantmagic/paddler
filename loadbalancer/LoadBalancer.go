package loadbalancer

import (
	"container/heap"
	"net/http"
	"net/url"

	"github.com/hashicorp/go-hclog"
)

type LoadBalancer struct {
	Logger       hclog.Logger
	LlamaCppHeap *LlamaCppHeap
}

func (self *LoadBalancer) Balance(request *http.Request) *url.URL {
	headTarget := self.LlamaCppHeap.Head()

	targetUrl := headTarget.
		LlamaCppClient.
		LlamaCppConfiguration.
		HttpAddress.
		BuildUrlWithPath("")

	headTarget.LlamaCppHealthStatus.SlotsIdle -= 1
	heap.Fix(self.LlamaCppHeap, 0)

	self.Logger.Debug("balancing", "target", targetUrl)

	return targetUrl
}
