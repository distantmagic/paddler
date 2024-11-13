package management

import "github.com/distantmagic/paddler/loadbalancer"

type RespondToDashboardTemplateProps struct {
	LlamaCppTargets    []*loadbalancer.LlamaCppTarget
	LoadBalancerStatus *loadbalancer.LoadBalancerStatus
}
