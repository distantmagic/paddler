package loadbalancer

import "time"

type LoadBalancerConfiguration struct {
	BalancingTimeoutDuration time.Duration
	BufferDriver             string
}

func (self *LoadBalancerConfiguration) IsBufferEnabled() bool {
	return self.BufferDriver != "none"
}
