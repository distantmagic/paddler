package loadbalancer

import "net/http"

type LoadBalancerRequest struct {
	BufferIfNoTargetsAvailable bool
	HttpRequest                *http.Request
}
