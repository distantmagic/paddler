package loadbalancer

import "net/http"

type LoadBalancerRequest struct {
	HttpRequest *http.Request
}
