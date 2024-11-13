package loadbalancer

import (
	"encoding/json"
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
)

type RespondToAggregatedHealth struct {
	LlamaCppHealthStatusAggregate *LlamaCppHealthStatusAggregate
	ServerEventsChannel           chan<- goroutine.ResultMessage
}

func (self *RespondToAggregatedHealth) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	mutexToken := self.LlamaCppHealthStatusAggregate.RBMutex.RLock()
	defer self.LlamaCppHealthStatusAggregate.RBMutex.RUnlock(mutexToken)

	jsonLoadBalancerStatus, err := json.Marshal(self.LlamaCppHealthStatusAggregate.AggregatedHealthStatus)

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
