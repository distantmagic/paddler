package loadbalancer

import "net/url"

type BalancingAttemptStatus struct {
	Error     error
	TargetUrl *url.URL
}
