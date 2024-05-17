package management

import (
	"encoding/json"
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/loadbalancer"
)

type RespondToHealth struct {
	LoadBalancer        *loadbalancer.LoadBalancer
	ServerEventsChannel chan<- goroutine.ResultMessage
}

func (self *RespondToHealth) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	loadBalancerStatus := self.LoadBalancer.GetStatus()

	jsonLoadBalancerStatus, err := json.Marshal(loadBalancerStatus)

	if err != nil {
		http.Error(response, err.Error(), http.StatusInternalServerError)

		return
	}

	response.Header().Set("Content-Type", "application/json")
	response.WriteHeader(http.StatusOK)

	_, err = response.Write(jsonLoadBalancerStatus)

	if err != nil {
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Error: err,
		}

		return
	}
}
