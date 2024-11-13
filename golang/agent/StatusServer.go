package agent

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/hashicorp/go-hclog"
)

type StatusServer struct {
	StatusServerConfiguration *StatusServerConfiguration
	Logger                    hclog.Logger
	RespondToHealth           http.Handler
}

func (self *StatusServer) Serve(serverEventsChannel chan<- goroutine.ResultMessage) {
	self.Logger.Debug(
		"listen",
		"host", self.StatusServerConfiguration.HttpAddress.GetHostWithPort(),
	)

	mux := http.NewServeMux()

	mux.Handle("/health", self.RespondToHealth)

	err := http.ListenAndServe(
		self.StatusServerConfiguration.HttpAddress.GetHostWithPort(),
		mux,
	)

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to listen",
			Error:   err,
		}
	}
}
