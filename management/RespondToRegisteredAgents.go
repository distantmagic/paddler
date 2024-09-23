package management

import (
	"encoding/json"
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/loadbalancer"
)

type RespondToRegisteredAgents struct {
	LoadBalancerTargetCollection *loadbalancer.LoadBalancerTargetCollection
	ServerEventsChannel 		 chan<- goroutine.ResultMessage
}

func (self *RespondToRegisteredAgents) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	registeredAgentsStatusJson, err := json.Marshal(self.LoadBalancerTargetCollection)

	if err != nil {
		http.Error(response, err.Error(), http.StatusInternalServerError)

		return
	}

	response.Header().Set("Content-Type", "application/json")
	response.WriteHeader(http.StatusOK)

	_, err = response.Write(registeredAgentsStatusJson)

	if err != nil {
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Error: err,
		}

		return
	}
}
