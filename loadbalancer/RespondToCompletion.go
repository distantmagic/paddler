package loadbalancer

import (
	"net/http"
	"time"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/hashicorp/go-hclog"
)

type RespondToCompletion struct {
	BufferChannel             chan *BufferedRequest
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

	bufferedRequestDoneChannel := make(chan bool)

	defer close(bufferedRequestDoneChannel)

	bufferedRequest := self.handleRequest(response, request, bufferedRequestDoneChannel)

	if bufferedRequest == nil {
		return
	}

	self.Logger.Info("BUFFER AWAITS", "url", request.URL.String())

	select {
	case <-bufferedRequest.Done:
		self.Logger.Info("BUFFER DONE", "url", request.URL.String())
		return
	case <-request.Context().Done():
		self.Logger.Info("Request DONE", "url", request.URL.String())
		return
	}
}

func (self *RespondToCompletion) bufferRequest(
	response http.ResponseWriter,
	request *http.Request,
	bufferedRequestDoneChannel chan bool,
) *BufferedRequest {
	bufferedRequest := &BufferedRequest{
		Done:     bufferedRequestDoneChannel,
		Response: response,
		Request:  request,
		Timeout:  time.Now().Add(self.LoadBalancerConfiguration.RequestBufferTimeout),
	}

	select {
	case self.BufferChannel <- bufferedRequest:
		return bufferedRequest
	default:
		http.Error(response, "Too Many Requests", http.StatusTooManyRequests)

		return nil
	}
}

func (self *RespondToCompletion) handleRequest(
	response http.ResponseWriter,
	request *http.Request,
	bufferedRequestDoneChannel chan bool,
) *BufferedRequest {
	balancingAttemptStatusChannel := make(chan *BalancingAttemptStatus)
	defer close(balancingAttemptStatusChannel)

	go self.LoadBalancer.Balance(
		balancingAttemptStatusChannel,
		&LoadBalancerRequest{
			HttpRequest: request,
		},
	)

	balancingAttemptStatus := <-balancingAttemptStatusChannel

	switch balancingAttemptStatus.Error {
	case ErrorNoTargetsAvailable:
		http.Error(response, "No Targets Available", http.StatusTooManyRequests)
	case ErrorNoSlotsAvailable:
		return self.bufferRequest(response, request, bufferedRequestDoneChannel)
	case nil:
		balancingAttemptStatus.LlamaCppTarget.ReverseProxy.ServeHTTP(response, request)
		bufferedRequestDoneChannel <- true
	default:
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Comment: "error while balancing request",
			Error:   balancingAttemptStatus.Error,
		}
	}

	return nil
}

func (self *RespondToCompletion) StartBufferProcessor() {
	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	for range ticker.C {
		select {
		case bufferedRequest := <-self.BufferChannel:
			if time.Now().After(bufferedRequest.Timeout) {
				http.Error(bufferedRequest.Response, "Request Timeout", http.StatusRequestTimeout)
				continue
			}

			self.handleRequest(
				bufferedRequest.Response,
				bufferedRequest.Request,
				bufferedRequest.Done,
			)
		default:
			// no buffered requests
		}
	}
}
