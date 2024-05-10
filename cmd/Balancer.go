package cmd

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/loadbalancer"
	"github.com/distantmagic/paddler/management"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
	"github.com/urfave/cli/v2"
)

type Balancer struct {
	Logger                        hclog.Logger
	ManagementServerConfiguration *management.ManagementServerConfiguration
	ReverseProxyConfiguration     *reverseproxy.ReverseProxyConfiguration
}

func (self *Balancer) Action(cliContext *cli.Context) error {
	serverEventsChannel := make(chan goroutine.ResultMessage)

	loadBalancer := loadbalancer.NewLoadBalancer(
		http.DefaultClient,
		self.Logger.Named("loadbalancer"),
	)

	managementServer := &management.Server{
		ManagementServerConfiguration: self.ManagementServerConfiguration,
		Logger:                        self.Logger.Named("management"),
		RespondToHealth:               &management.RespondToHealth{},
	}

	reverseProxyServer := &reverseproxy.Server{
		LoadBalancer:              loadBalancer,
		Logger:                    self.Logger.Named("reverseproxy"),
		ReverseProxyConfiguration: self.ReverseProxyConfiguration,
	}

	go loadBalancer.RegisterTarget(
		serverEventsChannel,
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

	go loadBalancer.RegisterTarget(
		serverEventsChannel,
		&loadbalancer.LlamaCppTargetConfiguration{
			LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
				HttpAddress: &netcfg.HttpAddressConfiguration{
					Host:   "127.0.0.1",
					Port:   8089,
					Scheme: "http",
				},
			},
		},
	)

	go managementServer.Serve(serverEventsChannel)
	go reverseProxyServer.Serve(serverEventsChannel)

	for serverEvent := range serverEventsChannel {
		self.Logger.Info(
			"server",
			"event", serverEvent,
		)
	}

	return nil
}
