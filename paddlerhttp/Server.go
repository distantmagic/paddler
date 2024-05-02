package paddlerhttp

import (
	"github.com/distantmagic/paddler/netcfg"
	"github.com/fasthttp/router"
	"github.com/hashicorp/go-hclog"
	"github.com/valyala/fasthttp"
)

type Server struct {
	HttpAddress   *netcfg.HttpAddressConfiguration
	Logger        hclog.Logger
	RespondToList *RespondToList
}

func (self *Server) Serve(serverEventsChannel chan ServerEvent) {
	defer close(serverEventsChannel)

	self.Logger.Debug("serve")

	routes := router.New()
	routes.GET("/list", self.RespondToList.CreateResponse)

	err := fasthttp.ListenAndServe(
		self.HttpAddress.GetHostWithPort(),
		routes.Handler,
	)

	if err != nil {
		serverEventsChannel <- ServerEvent{
			Error: err,
		}
	}
}
