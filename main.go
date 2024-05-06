package main

import (
	"container/heap"
	"flag"
	"net/http"

	"github.com/distantmagic/paddler/httpserver"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/loadbalancer"
	"github.com/distantmagic/paddler/metahttp"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
)

var (
	FlagMetaHost           = flag.String("paddler-host", "127.0.0.1", "Meta host to bind to")
	FlagMetaPort           = flag.Uint("paddler-port", 8082, "Meta port to bind to")
	FlagMetaScheme         = flag.String("paddler-scheme", "http", "Meta scheme to use")
	FlagReverseProxyHost   = flag.String("reverseproxy-host", "127.0.0.1", "Reverse proxy host to bind to")
	FlagReverseProxyPort   = flag.Uint("reverseproxy-port", 8083, "Reverse proxy port to bind to")
	FlagReverseProxyScheme = flag.String("reverseproxy-scheme", "http", "Reve rseproxy scheme to use")
)

func main() {
	flag.Parse()

	logger := hclog.New(&hclog.LoggerOptions{
		Name:  "paddler",
		Level: hclog.Debug,
	})

	serverEventsChannel := make(chan httpserver.ServerEvent)

	paddlerHttpServer := &metahttp.Server{
		HttpAddress: &netcfg.HttpAddressConfiguration{
			Host:   *FlagMetaHost,
			Port:   *FlagMetaPort,
			Scheme: *FlagMetaScheme,
		},
		Logger:          logger.Named("metahttp.Server"),
		RespondToHealth: &metahttp.RespondToHealth{},
	}

	llamaCppHeap := &loadbalancer.LlamaCppHeap{}

	heap.Init(llamaCppHeap)
	heap.Push(llamaCppHeap, &loadbalancer.LlamaCppTarget{
		LlamaCppClient: &llamacpp.LlamaCppClient{
			HttpClient: http.DefaultClient,
			LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
				HttpAddress: &netcfg.HttpAddressConfiguration{
					Host:   "127.0.0.1",
					Port:   8088,
					Scheme: "http",
				},
			},
		},
		LlamaCppHealthStatus: &llamacpp.LlamaCppHealthStatus{
			SlotsIdle: 10,
		},
	})
	heap.Push(llamaCppHeap, &loadbalancer.LlamaCppTarget{
		LlamaCppClient: &llamacpp.LlamaCppClient{
			HttpClient: http.DefaultClient,
			LlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
				HttpAddress: &netcfg.HttpAddressConfiguration{
					Host:   "127.0.0.1",
					Port:   8089,
					Scheme: "http",
				},
			},
		},
		LlamaCppHealthStatus: &llamacpp.LlamaCppHealthStatus{
			SlotsIdle: 10,
		},
	})

	loadBalancer := &loadbalancer.LoadBalancer{
		Logger:       logger.Named("LoadBalancer"),
		LlamaCppHeap: llamaCppHeap,
	}

	reverseProxyServer := &reverseproxy.Server{
		HttpAddress: &netcfg.HttpAddressConfiguration{
			Host:   *FlagReverseProxyHost,
			Port:   *FlagReverseProxyPort,
			Scheme: *FlagReverseProxyScheme,
		},
		LoadBalancer: loadBalancer,
		Logger:       logger.Named("reverseproxy.Server"),
	}

	go paddlerHttpServer.Serve(serverEventsChannel)
	go reverseProxyServer.Serve(serverEventsChannel)

	for serverEvent := range serverEventsChannel {
		logger.Info("server event", serverEvent)
	}
}
