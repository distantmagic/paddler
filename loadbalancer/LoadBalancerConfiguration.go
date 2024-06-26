package loadbalancer

import "time"

type LoadBalancerConfiguration struct {
	RequestBufferSize    uint
	RequestBufferTimeout time.Duration
}
