package loadbalancer

import (
	"encoding/json"
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
)

type RespondToAggregatedHealth struct {
	LoadBalancerTargetCollection *LoadBalancerTargetCollection
	ServerEventsChannel          chan<- goroutine.ResultMessage
}

func (self *RespondToAggregatedHealth) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	jsonLoadBalancerStatus, err := json.Marshal(self.LoadBalancerTargetCollection.AggregatedHealthStatus)

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
