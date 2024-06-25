package loadbalancer

import "net/http"

type LoadBalancerRequest struct {
	BufferIfNoTargetsAvailable bool
	HttpRequest                *http.Request
}

func (self *LoadBalancerRequest) IsSlottable() bool {
	return self.HttpRequest.Method == "POST" && self.HttpRequest.URL.Path == "/completion"
}
