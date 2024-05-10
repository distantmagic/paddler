package management

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/hashicorp/go-hclog"
)

type Server struct {
	ManagementServerConfiguration *ManagementServerConfiguration
	Logger                        hclog.Logger
	RespondToHealth               *RespondToHealth
}

func (self *Server) Serve(serverEventsChannel chan goroutine.ResultMessage) {
	self.Logger.Debug(
		"listen",
		"host", self.ManagementServerConfiguration.HttpAddress.GetHostWithPort(),
	)

	mux := http.NewServeMux()

	mux.Handle("/health", self.RespondToHealth)

	err := http.ListenAndServe(
		self.ManagementServerConfiguration.HttpAddress.GetHostWithPort(),
		mux,
	)

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to listen",
			Error: err,
		}
	}
}
