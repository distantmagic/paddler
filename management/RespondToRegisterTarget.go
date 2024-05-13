package management

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/loadbalancer"
)

type RespondToRegisterTarget struct {
	LoadBalancer        *loadbalancer.LoadBalancer
	ServerEventsChannel chan goroutine.ResultMessage
}

func (self *RespondToRegisterTarget) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	if request.Method != http.MethodPost {
		http.Error(response, "Only POST method is allowed", http.StatusMethodNotAllowed)

		return
	}

	var registerTargetRequest RegisterTargetRequest

	decoder := json.NewDecoder(request.Body)

	err := decoder.Decode(&registerTargetRequest)

	if err != nil {
		http.Error(response, err.Error(), http.StatusBadRequest)

		self.ServerEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to decode request body",
			Error:   err,
		}

		return
	}

	go self.LoadBalancer.RegisterTarget(
		self.ServerEventsChannel,
		registerTargetRequest.LlamaCppTargetConfiguration,
	)

	response.Header().Set("Content-Type", "application/json")
	response.WriteHeader(http.StatusOK)
	fmt.Fprintf(response, `{"status":"ok"}`)
}
