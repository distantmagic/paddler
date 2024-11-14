package loadbalancer

type LoadBalancerStatus struct {
	RegisteredTargets int `json:"registered_targets"`
}
