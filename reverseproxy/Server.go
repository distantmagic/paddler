package reverseproxy

import (
	"net/http"
	"net/http/httputil"

	"github.com/distantmagic/paddler/httpserver"
	"github.com/distantmagic/paddler/loadbalancer"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/hashicorp/go-hclog"
)

type Server struct {
	HttpAddress  *netcfg.HttpAddressConfiguration
	LoadBalancer *loadbalancer.LoadBalancer
	Logger       hclog.Logger
}

func (self *Server) Serve(serverEventsChannel chan httpserver.ServerEvent) {
	self.Logger.Debug("serve")

	reverseProxyController := &ReverseProxyController{
		Logger: self.Logger.Named("ReverseProxyController"),
		ReverseProxy: &httputil.ReverseProxy{
			Rewrite: func(request *httputil.ProxyRequest) {
				request.SetURL(self.LoadBalancer.Balance(request.In))
				request.SetXForwarded()
			},
		},
	}

	err := http.ListenAndServe(":8083", reverseProxyController)

	if err != nil {
		serverEventsChannel <- httpserver.ServerEvent{
			Error: err,
		}
	}
}
