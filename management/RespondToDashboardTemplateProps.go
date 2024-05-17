package management

import "github.com/distantmagic/paddler/loadbalancer"

type RespondToDashboardTemplateProps struct {
	LlamaCppTargets    <-chan *loadbalancer.LlamaCppTarget
	LoadBalancerStatus *loadbalancer.LoadBalancerStatus
}
