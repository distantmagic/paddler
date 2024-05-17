package cmd

import (
	"net/http"
	"time"

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

	respondToDashboard, err := management.NewRespondToDashboard(
		loadBalancer,
		serverEventsChannel,
	)

	if err != nil {
		return err
	}

	managementServer := &management.Server{
		ManagementServerConfiguration: self.ManagementServerConfiguration,
		Logger:                        self.Logger.Named("management"),
		RespondToDashboard:            respondToDashboard,
		RespondToHealth: &management.RespondToHealth{
			LoadBalancer:        loadBalancer,
			ServerEventsChannel: serverEventsChannel,
		},
		RespondToRegisterTarget: &management.RespondToRegisterTarget{
			LoadBalancer:        loadBalancer,
			ServerEventsChannel: serverEventsChannel,
		},
		RespondToStatic: management.NewRespondToStatic(),
	}

	reverseProxyServer := &loadbalancer.ReverseProxyServer{
		LoadBalancer:              loadBalancer,
		Logger:                    self.Logger.Named("reverseproxy"),
		ReverseProxyConfiguration: self.ReverseProxyConfiguration,
	}

	go managementServer.Serve(serverEventsChannel)
	go reverseProxyServer.Serve(serverEventsChannel)

	ticker := time.NewTicker(time.Second * 1)

	go self.RuntTickerInterval(
		ticker,
		serverEventsChannel,
		loadBalancer,
	)

	for serverEvent := range serverEventsChannel {
		self.Logger.Log(
			hclog.Info,
			"server event",
			"event", serverEvent,
		)
	}

	return nil
}

func (self *Balancer) RuntTickerInterval(
	ticker *time.Ticker,
	serverEventsChannel chan<- goroutine.ResultMessage,
	loadBalancer *loadbalancer.LoadBalancer,
) {
	for range ticker.C {
		go loadBalancer.OnTick()
	}
}
