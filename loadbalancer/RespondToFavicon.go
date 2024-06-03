package loadbalancer

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
)

type RespondToFavicon struct {
	ServerEventsChannel chan<- goroutine.ResultMessage
}

func (self *RespondToFavicon) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	response.WriteHeader(http.StatusNotFound)

	_, err := response.Write([]byte("404 - Not Found"))

	if err != nil {
		self.ServerEventsChannel <- goroutine.ResultMessage{
			Error: err,
		}
	}
}
