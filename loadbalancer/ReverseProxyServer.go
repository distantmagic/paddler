package loadbalancer

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
)

type ReverseProxyServer struct {
	Logger                    hclog.Logger
	RespondToAggregatedHealth *RespondToAggregatedHealth
	RespondToCompletion       *RespondToCompletion
	RespondToFavicon          *RespondToFavicon
	RespondToFallback         *RespondToFallback
	ReverseProxyConfiguration *reverseproxy.ReverseProxyConfiguration
}

func (self *ReverseProxyServer) Serve(serverEventsChannel chan<- goroutine.ResultMessage) {
	self.Logger.Debug(
		"listen",
		"host", self.ReverseProxyConfiguration.HttpAddress.GetHostWithPort(),
	)

	reverseProxyMux := http.NewServeMux()
	reverseProxyMux.Handle("/favicon.ico", self.RespondToFavicon)
	reverseProxyMux.Handle("/health", self.RespondToAggregatedHealth)
	reverseProxyMux.Handle("/completion", self.RespondToCompletion)
	reverseProxyMux.Handle("/", self.RespondToFallback)

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
