package loadbalancer

import (
	"net/http"
)

type RespondToFallback struct {
	LoadBalancerTargetCollection *LoadBalancerTargetCollection
}

func (self *RespondToFallback) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	headPickedTarget := self.LoadBalancerTargetCollection.GetHeadTarget()

	if headPickedTarget == nil {
		http.Error(response, "No Targets Available", http.StatusBadGateway)
		return
	}

	headPickedTarget.LlamaCppTarget.ReverseProxy.ServeHTTP(response, request)
}
