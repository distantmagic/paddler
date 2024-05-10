package cmd

import (
	"github.com/distantmagic/paddler/httpserver"
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
	serverEventsChannel := make(chan httpserver.ServerEvent)

	loadBalancer := loadbalancer.NewLoadBalancer(
		self.Logger.Named("loadbalancer"),
	)

	loadBalancer.RegisterTarget(&loadbalancer.LlamaCppTargetConfiguration{
		LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{
				Host:   "127.0.0.1",
				Port:   8088,
				Scheme: "http",
			},
		},
	})

	// loadBalancer.RegisterTarget(&loadbalancer.LlamaCppTarget{
	// 	LlamaCppClient: &llamacpp.LlamaCppClient{
	// 		HttpClient: http.DefaultClient,
	// 		LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
	// 			HttpAddress: &netcfg.HttpAddressConfiguration{
	// 				Host:   "127.0.0.1",
	// 				Port:   8089,
	// 				Scheme: "http",
	// 			},
	// 		},
	// 	},
	// 	LlamaCppHealthStatus: &llamacpp.LlamaCppHealthStatus{
	// 		SlotsIdle: 10,
	// 	},
	// })

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

	go managementServer.Serve(serverEventsChannel)
	go reverseProxyServer.Serve(serverEventsChannel)

	for serverEvent := range serverEventsChannel {
		self.Logger.Info("server event", serverEvent)
	}

	return nil
}
