package agent

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
)

type RespondToHealth struct {
	ServerEventsChannel chan<- goroutine.ResultMessage
}

func (self *RespondToHealth) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	response.Header().Set("Content-Type", "text/plain")
	response.WriteHeader(http.StatusOK)

	_, err := response.Write([]byte("OK"))

	if err != nil {
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Error: err,
		}

		return
	}
}
