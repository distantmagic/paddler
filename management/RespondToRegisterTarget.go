package management

import (
	"fmt"
	"html"
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/loadbalancer"
	"github.com/distantmagic/paddler/netcfg"
)

type RespondToRegisterTarget struct {
	LoadBalancer        *loadbalancer.LoadBalancer
	ServerEventsChannel chan goroutine.ResultMessage
}

func (self *RespondToRegisterTarget) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	go self.LoadBalancer.RegisterTarget(
		self.ServerEventsChannel,
		&loadbalancer.LlamaCppTargetConfiguration{
			LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
				HttpAddress: &netcfg.HttpAddressConfiguration{
					Host:   "127.0.0.1",
					Port:   8088,
					Scheme: "http",
				},
			},
		},
	)

	fmt.Fprintf(response, "Hello, %q", html.EscapeString(request.URL.Path))
}
