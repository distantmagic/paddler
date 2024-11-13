package loadbalancer

import (
	"net/http"
	"time"
)

type BufferedRequest struct {
	DoneChannel         chan bool
	IsDoneChannelClosed bool
	IsDone              bool
	Response            http.ResponseWriter
	Request             *http.Request
	Timeout             time.Time
}

func (self *BufferedRequest) Close() {
	if self.IsDoneChannelClosed {
		return
	}

	self.IsDone = true
	self.IsDoneChannelClosed = true

	close(self.DoneChannel)
}

func (self *BufferedRequest) SendError(message string, errorCode int) {
	if self.IsDone {
		return
	}

	http.Error(self.Response, message, errorCode)
	self.SetDone()
}

func (self *BufferedRequest) SetDone() {
	if self.IsDone {
		return
	}

	self.DoneChannel <- true
	self.IsDone = true
}
