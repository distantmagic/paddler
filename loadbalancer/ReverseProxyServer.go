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
	RespondToAggregatedHealth *RespondToAggregatedHealth
	RespondToFavicon          *RespondToFavicon
	ReverseProxyConfiguration *reverseproxy.ReverseProxyConfiguration
}

func (self *ReverseProxyServer) Serve(serverEventsChannel chan<- goroutine.ResultMessage) {
	self.Logger.Debug(
		"listen",
		"host", self.ReverseProxyConfiguration.HttpAddress.GetHostWithPort(),
	)

	reverseProxy := &httputil.ReverseProxy{
		ErrorLog: self.Logger.Named("ReverseProxy").StandardLogger(&hclog.StandardLoggerOptions{
			InferLevels: true,
		}),
		Rewrite: func(proxyRequest *httputil.ProxyRequest) {
			targetUrl, err := self.LoadBalancer.Balance(&LoadBalancerRequest{
				HttpRequest: proxyRequest.In,
			})

			if err == nil {
				proxyRequest.SetURL(targetUrl)
				proxyRequest.SetXForwarded()
			} else {
				serverEventsChannel <- goroutine.ResultMessage{
					Comment: "failed to balance request",
					Error:   err,
				}
			}
		},
	}

	reverseProxyMux := http.NewServeMux()
	reverseProxyMux.Handle("/favicon.ico", self.RespondToFavicon)
	reverseProxyMux.Handle("/health", self.RespondToAggregatedHealth)
	reverseProxyMux.Handle("/", reverseProxy)

	err := http.ListenAndServe(
		self.ReverseProxyConfiguration.HttpAddress.GetHostWithPort(),
		reverseProxyMux,
	)

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to listen",
			Error:   err,
		}
	}
}
