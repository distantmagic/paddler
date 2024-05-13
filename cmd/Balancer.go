package cmd

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/loadbalancer"
	"github.com/distantmagic/paddler/management"
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
		RespondToHealth: &management.RespondToHealth{
			LoadBalancer:        loadBalancer,
			ServerEventsChannel: serverEventsChannel,
		},
		RespondToRegisterTarget: &management.RespondToRegisterTarget{
			LoadBalancer:        loadBalancer,
			ServerEventsChannel: serverEventsChannel,
		},
	}

	reverseProxyServer := &loadbalancer.ReverseProxyServer{
		LoadBalancer:              loadBalancer,
		Logger:                    self.Logger.Named("reverseproxy"),
		ReverseProxyConfiguration: self.ReverseProxyConfiguration,
	}

	go managementServer.Serve(serverEventsChannel)
	go reverseProxyServer.Serve(serverEventsChannel)

	for serverEvent := range serverEventsChannel {
		self.Logger.Log(
			hclog.Info,
			"server event",
			"event", serverEvent,
		)
	}

	return nil
}
