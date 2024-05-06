package metahttp

import (
	"net/http"

	"github.com/distantmagic/paddler/httpserver"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/hashicorp/go-hclog"
)

type Server struct {
	HttpAddress     *netcfg.HttpAddressConfiguration
	Logger          hclog.Logger
	RespondToHealth *RespondToHealth
}

func (self *Server) Serve(serverEventsChannel chan httpserver.ServerEvent) {
	self.Logger.Debug("serve")

	mux := http.NewServeMux()

	mux.Handle("/health", self.RespondToHealth)

	err := http.ListenAndServe(
		self.HttpAddress.GetHostWithPort(),
		mux,
	)

	if err != nil {
		serverEventsChannel <- httpserver.ServerEvent{
			Error: err,
		}
	}
}
