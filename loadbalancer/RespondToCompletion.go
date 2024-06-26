package loadbalancer

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/hashicorp/go-hclog"
)

type RespondToCompletion struct {
	LoadBalancer              *LoadBalancer
	LoadBalancerConfiguration *LoadBalancerConfiguration
	Logger                    hclog.Logger
	RequestBuffer             *RequestBuffer
	ServerEventsChannel       chan<- goroutine.ResultMessage
}

func (self *RespondToCompletion) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	if request.Method != http.MethodPost {
		http.Error(response, "Method Not Allowed", http.StatusMethodNotAllowed)
		return
	}

	balancingAttemptStatusChannel := make(chan *BalancingAttemptStatus)

	defer close(balancingAttemptStatusChannel)

	go self.LoadBalancer.Balance(
		balancingAttemptStatusChannel,
		&LoadBalancerRequest{
			BufferIfNoTargetsAvailable: self.LoadBalancerConfiguration.IsBufferEnabled(),
			HttpRequest:                request,
		},
	)

	balancingAttemptStatus := <-balancingAttemptStatusChannel

	switch balancingAttemptStatus.Error {
	case ErrorNoTargetsAvailable:
		http.Error(response, "No Targets Available", http.StatusTooManyRequests)
	case ErrorNoSlotsAvailable:
		http.Error(response, "Too Many Requests", http.StatusTooManyRequests)
	case nil:
		balancingAttemptStatus.LlamaCppTarget.ReverseProxy.ServeHTTP(
			response,
			request,
		)
	default:
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Comment: "error while balancing request",
			Error:   balancingAttemptStatus.Error,
		}
	}
}
