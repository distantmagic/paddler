package loadbalancer

import (
	"net/http"
	"net/http/httputil"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
)

type ReverseProxyServer struct {
	LoadBalancer              *LoadBalancer
	Logger                    hclog.Logger
	ReverseProxyConfiguration *reverseproxy.ReverseProxyConfiguration
}

func (self *ReverseProxyServer) Serve(serverEventsChannel chan goroutine.ResultMessage) {
	self.Logger.Debug(
		"listen",
		"host", self.ReverseProxyConfiguration.HttpAddress.GetHostWithPort(),
	)

	reverseProxy := &httputil.ReverseProxy{
		ErrorLog: self.Logger.Named("ReverseProxy").StandardLogger(&hclog.StandardLoggerOptions{
			InferLevels: true,
		}),
		Rewrite: func(request *httputil.ProxyRequest) {
			targetUrl, err := self.LoadBalancer.Balance(request.In)

			if err != nil {
				serverEventsChannel <- goroutine.ResultMessage{
					Comment: "failed to balance request",
					Error:   err,
				}

				return
			}

			request.SetURL(targetUrl)
			request.SetXForwarded()
		},
	}

	err := http.ListenAndServe(
		self.ReverseProxyConfiguration.HttpAddress.GetHostWithPort(),
		reverseProxy,
	)

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to listen",
			Error:   err,
		}
	}
}
