package reverseproxy

import (
	"net/http"
	"net/http/httputil"

	"github.com/distantmagic/paddler/httpserver"
	"github.com/distantmagic/paddler/loadbalancer"
	"github.com/hashicorp/go-hclog"
)

type Server struct {
	LoadBalancer              *loadbalancer.LoadBalancer
	Logger                    hclog.Logger
	ReverseProxyConfiguration *ReverseProxyConfiguration
}

func (self *Server) Serve(serverEventsChannel chan httpserver.ServerEvent) {
	self.Logger.Debug(
		"listen",
		"host", self.ReverseProxyConfiguration.HttpAddress.GetHostWithPort(),
	)

	reverseProxyController := &ReverseProxyController{
		Logger: self.Logger.Named("ReverseProxyController"),
		ReverseProxy: &httputil.ReverseProxy{
			Rewrite: func(request *httputil.ProxyRequest) {
				targetUrl, err := self.LoadBalancer.Balance(request.In)

				if err != nil {
					panic(err)
				}

				request.SetURL(targetUrl)
				request.SetXForwarded()
			},
		},
	}

	err := http.ListenAndServe(
		self.ReverseProxyConfiguration.HttpAddress.GetHostWithPort(),
		reverseProxyController,
	)

	if err != nil {
		serverEventsChannel <- httpserver.ServerEvent{
			Error: err,
		}
	}
}
