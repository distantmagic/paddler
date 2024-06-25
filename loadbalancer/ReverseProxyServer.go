package loadbalancer

import (
	"context"
	"net/http"
	"net/http/httputil"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
)

type ReverseProxyServer struct {
	LoadBalancer              *LoadBalancer
	LoadBalancerConfiguration *LoadBalancerConfiguration
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
			balancingAttemptStatusChannel := make(chan *BalancingAttemptStatus)

			defer close(balancingAttemptStatusChannel)

			ctx, cancel := context.WithTimeout(
				context.Background(),
				self.LoadBalancerConfiguration.BalancingTimeoutDuration,
			)

			defer cancel()

			go self.LoadBalancer.Balance(
				ctx,
				balancingAttemptStatusChannel,
				&LoadBalancerRequest{
					BufferIfNoTargetsAvailable: self.LoadBalancerConfiguration.IsBufferEnabled(),
					HttpRequest:                proxyRequest.In,
				},
			)

			select {
			case <-ctx.Done():
				serverEventsChannel <- goroutine.ResultMessage{
					Comment: "balancing a request timed out",
					Error:   ctx.Err(),
				}

				return
			case balancingAttemptStatus := <-balancingAttemptStatusChannel:
				if balancingAttemptStatus.Error == nil {
					proxyRequest.SetURL(balancingAttemptStatus.TargetUrl)
					proxyRequest.SetXForwarded()
				} else {
					serverEventsChannel <- goroutine.ResultMessage{
						Comment: "error while balancing request",
						Error:   balancingAttemptStatus.Error,
					}
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
