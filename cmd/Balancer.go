package cmd

import (
	"errors"
	"net/http"
	"time"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/loadbalancer"
	"github.com/distantmagic/paddler/management"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
	statsd "github.com/smira/go-statsd"
	"github.com/urfave/cli/v2"
)

var (
	ErrorUnrecognizedBufferDriver = errors.New("unrecognized queue driver")
)

type Balancer struct {
	LoadBalancerConfiguration     *loadbalancer.LoadBalancerConfiguration
	Logger                        hclog.Logger
	ManagementServerConfiguration *management.ManagementServerConfiguration
	ReverseProxyConfiguration     *reverseproxy.ReverseProxyConfiguration
	StatsdConfiguration           *loadbalancer.StatsdConfiguration
}

func (self *Balancer) Action(cliContext *cli.Context) error {
	serverEventsChannel := make(chan goroutine.ResultMessage)

	defer close(serverEventsChannel)

	requestBuffer, err := self.MakeRequestBuffer()

	if err != nil {
		return err
	}

	llamaCppHealthStatusAggregate := &loadbalancer.LlamaCppHealthStatusAggregate{
		AggregatedHealthStatus: &llamacpp.LlamaCppHealthStatus{
			Status: llamacpp.Ok,
		},
	}

	loadBalancerTargetCollection := loadbalancer.NewLoadBalancerTargetCollection(llamaCppHealthStatusAggregate)

	loadBalancer := &loadbalancer.LoadBalancer{
		LoadBalancerTargetCollection: loadBalancerTargetCollection,
		Logger:                       self.Logger,
		RequestBuffer:                &requestBuffer,
	}

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
			LoadBalancerTargetRegistrar: &loadbalancer.LoadBalancerTargetRegistrar{
				HttpClient:                   http.DefaultClient,
				LoadBalancerTargetCollection: loadBalancer.LoadBalancerTargetCollection,
				Logger:                       self.Logger,
			},
			ServerEventsChannel: serverEventsChannel,
		},
		RespondToStatic: management.NewRespondToStatic(),
	}

	reverseProxyServer := &loadbalancer.ReverseProxyServer{
		LoadBalancer:              loadBalancer,
		LoadBalancerConfiguration: self.LoadBalancerConfiguration,
		Logger:                    self.Logger.Named("reverseproxy"),
		RespondToAggregatedHealth: &loadbalancer.RespondToAggregatedHealth{
			LlamaCppHealthStatusAggregate: llamaCppHealthStatusAggregate,
			ServerEventsChannel:           serverEventsChannel,
		},
		RespondToFavicon:          &loadbalancer.RespondToFavicon{},
		ReverseProxyConfiguration: self.ReverseProxyConfiguration,
	}

	go managementServer.Serve(serverEventsChannel)
	go reverseProxyServer.Serve(serverEventsChannel)

	ticker := time.NewTicker(time.Second * 1)

	go self.RuntTickerInterval(
		ticker,
		serverEventsChannel,
		&loadbalancer.LoadBalancerTemporalManager{
			LlamaCppHealthStatusAggregate: llamaCppHealthStatusAggregate,
			LoadBalancerTargetCollection:  loadBalancerTargetCollection,
			ServerEventsChannel:           serverEventsChannel,
			StatsdReporter:                self.MakeStatsdReporter(),
		},
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

func (self *Balancer) MakeRequestBuffer() (loadbalancer.RequestBuffer, error) {
	var requestBuffer loadbalancer.RequestBuffer

	switch self.LoadBalancerConfiguration.BufferDriver {
	case "memory":
		requestBuffer = &loadbalancer.MemoryRequestBuffer{}
	case "none":
		requestBuffer = &loadbalancer.VoidRequestBuffer{}
	default:
		return nil, ErrorUnrecognizedBufferDriver
	}

	return requestBuffer, nil
}

func (self *Balancer) MakeStatsdReporter() loadbalancer.StatsdReporterInterface {
	var statsdReporter loadbalancer.StatsdReporterInterface

	if self.StatsdConfiguration.EnableStatsdReporter {
		self.Logger.Log(
			hclog.Info,
			"starting statsd reporter",
			"target host", self.StatsdConfiguration.HttpAddress.GetHostWithPort(),
		)
		statsdReporter = &loadbalancer.StatsdReporter{
			StatsdClient: *statsd.NewClient(
				self.StatsdConfiguration.HttpAddress.GetHostWithPort(),
				statsd.MetricPrefix("paddler."),
			),
		}
	} else {
		statsdReporter = &loadbalancer.StatsdReporterVoid{}
	}

	return statsdReporter
}

func (self *Balancer) RuntTickerInterval(
	ticker *time.Ticker,
	serverEventsChannel chan<- goroutine.ResultMessage,
	loadBalancerTemporalManager *loadbalancer.LoadBalancerTemporalManager,
) {
	for range ticker.C {
		go loadBalancerTemporalManager.OnApplicationTick()
	}
}
