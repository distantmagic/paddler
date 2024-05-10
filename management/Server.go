package management

import (
	"net/http"

	"github.com/distantmagic/paddler/httpserver"
	"github.com/hashicorp/go-hclog"
)

type Server struct {
	ManagementServerConfiguration *ManagementServerConfiguration
	Logger                        hclog.Logger
	RespondToHealth               *RespondToHealth
}

func (self *Server) Serve(serverEventsChannel chan httpserver.ServerEvent) {
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
		serverEventsChannel <- httpserver.ServerEvent{
			Error: err,
		}
	}
}
