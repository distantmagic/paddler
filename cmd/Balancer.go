package cmd

import (
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/loadbalancer"
	"github.com/distantmagic/paddler/management"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
	statsd "github.com/smira/go-statsd"
	"github.com/urfave/cli/v2"
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

	llamaCppHealthStatusAggregate := &loadbalancer.LlamaCppHealthStatusAggregate{
		AggregatedHealthStatus: &llamacpp.LlamaCppSlotsAggregatedStatus{
			Status: llamacpp.Ok,
		},
	}

	loadBalancerTargetCollection := loadbalancer.NewLoadBalancerTargetCollection(llamaCppHealthStatusAggregate)

	loadBalancer := &loadbalancer.LoadBalancer{
		LoadBalancerTargetCollection: loadBalancerTargetCollection,
		Logger:                       self.Logger,
	}

	respondToDashboard, err := management.NewRespondToDashboard(
		loadBalancer,
		serverEventsChannel,
	)

	if err != nil {
		return err
	}

	bufferedRequestsStats := &loadbalancer.BufferedRequestsStats{}

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

	respondToCompletion := &loadbalancer.RespondToCompletion{
		BufferChannel:             make(chan *loadbalancer.BufferedRequest, self.LoadBalancerConfiguration.RequestBufferSize),
		BufferedRequestsStats:     bufferedRequestsStats,
		LoadBalancer:              loadBalancer,
		LoadBalancerConfiguration: self.LoadBalancerConfiguration,
		Logger:                    self.Logger.Named("respond_to_completion"),
	}

	reverseProxyServer := &loadbalancer.ReverseProxyServer{
		Logger: self.Logger.Named("reverseproxy"),
		RespondToAggregatedHealth: &loadbalancer.RespondToAggregatedHealth{
			LlamaCppHealthStatusAggregate: llamaCppHealthStatusAggregate,
			ServerEventsChannel:           serverEventsChannel,
		},
		RespondToCompletion: respondToCompletion,
		RespondToFallback: &loadbalancer.RespondToFallback{
			LoadBalancerTargetCollection: loadBalancerTargetCollection,
		},
		RespondToFavicon:          &loadbalancer.RespondToFavicon{},
		ReverseProxyConfiguration: self.ReverseProxyConfiguration,
	}

	loadBalancerTemporalManager := &loadbalancer.LoadBalancerTemporalManager{
		BufferedRequestsStats:         bufferedRequestsStats,
		LlamaCppHealthStatusAggregate: llamaCppHealthStatusAggregate,
		LoadBalancerTargetCollection:  loadBalancerTargetCollection,
		StatsdReporter:                self.MakeStatsdReporter(),
	}

	go loadBalancerTemporalManager.RunTickerInterval()
	go managementServer.Serve(serverEventsChannel)
	go respondToCompletion.StartBufferProcessor()
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
