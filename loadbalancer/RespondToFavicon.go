package loadbalancer

import (
	"net/http"
)

type RespondToFavicon struct{}

func (self *RespondToFavicon) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	response.WriteHeader(http.StatusNotFound)
	response.Write([]byte("404 - Not Found"))
}
