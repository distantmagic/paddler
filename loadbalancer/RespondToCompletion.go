package loadbalancer

import (
	"net/http"
	"time"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/hashicorp/go-hclog"
)

type RespondToCompletion struct {
	BufferChannel             chan *BufferedRequest
	BufferedRequestsStats     *BufferedRequestsStats
	LoadBalancer              *LoadBalancer
	LoadBalancerConfiguration *LoadBalancerConfiguration
	Logger                    hclog.Logger
	ServerEventsChannel       chan<- goroutine.ResultMessage
}

func (self *RespondToCompletion) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	if request.Method != http.MethodPost {
		http.Error(response, "Method Not Allowed", http.StatusMethodNotAllowed)
		return
	}

	bufferedRequest := &BufferedRequest{
		DoneChannel: make(chan bool),
		Response:    response,
		Request:     request,
		Timeout:     time.Now().Add(self.LoadBalancerConfiguration.RequestBufferTimeout),
	}

	defer bufferedRequest.Close()

	go self.handleRequest(bufferedRequest, true)

	select {
	case <-bufferedRequest.DoneChannel:
		// buffer finished
	case <-request.Context().Done():
		// request is canceled
	case <-time.After(self.LoadBalancerConfiguration.RequestBufferTimeout):
		http.Error(response, "Request Timeout", http.StatusGatewayTimeout)
	}
}

func (self *RespondToCompletion) bufferRequest(bufferedRequest *BufferedRequest) {
	select {
	case self.BufferChannel <- bufferedRequest:
	default:
		bufferedRequest.SendError("Too Many Requests", http.StatusTooManyRequests)
	}
}

func (self *RespondToCompletion) handleRequest(
	bufferedRequest *BufferedRequest,
	isInitial bool,
) {
	balancingAttemptStatusChannel := make(chan *BalancingAttemptStatus)
	defer close(balancingAttemptStatusChannel)

	go self.LoadBalancer.Balance(
		balancingAttemptStatusChannel,
		&LoadBalancerRequest{
			HttpRequest: bufferedRequest.Request,
		},
	)

	balancingAttemptStatus := <-balancingAttemptStatusChannel

	switch balancingAttemptStatus.Error {
	case ErrorNoTargetsAvailable:
		bufferedRequest.SendError("No Targets Available", http.StatusServiceUnavailable)
	case ErrorNoSlotsAvailable:
		if isInitial {
			// do not increase buffered requests stat multiple times
			self.BufferedRequestsStats.RequestsBuffered += 1
		}

		go self.bufferRequest(bufferedRequest)
	case nil:
		balancingAttemptStatus.LlamaCppTarget.ReverseProxy.ServeHTTP(
			bufferedRequest.Response,
			bufferedRequest.Request,
		)
		bufferedRequest.SetDone()
	default:
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Comment: "error while balancing request",
			Error:   balancingAttemptStatus.Error,
		}
	}
}

func (self *RespondToCompletion) StartBufferProcessor() {
	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	for range ticker.C {
		select {
		case bufferedRequest := <-self.BufferChannel:
			if bufferedRequest.IsDone {
				continue
			}

			if time.Now().After(bufferedRequest.Timeout) {
				continue
			}

			go self.handleRequest(bufferedRequest, false)
		default:
			// no buffered requests
		}
	}
}
