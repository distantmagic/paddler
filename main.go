package main

import (
	"os"

	"github.com/distantmagic/paddler/cmd"
	"github.com/distantmagic/paddler/management"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
	"github.com/urfave/cli/v2"
)

func main() {
	logger := hclog.New(&hclog.LoggerOptions{
		Name:  "paddler",
		Level: hclog.Debug,
	})

	agent := &cmd.Agent{
		Logger: logger.Named("Agent"),
	}

	balancer := &cmd.Balancer{
		Logger: logger.Named("Balancer"),
		ManagementServerConfiguration: &management.ManagementServerConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{},
		},
		ReverseProxyConfiguration: &reverseproxy.ReverseProxyConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{},
		},
	}

	app := &cli.App{
		Name:  "paddler",
		Usage: "llama.cpp load balaner and reverse proxy server",
		Commands: []*cli.Command{
			{
				Name:   "agent",
				Usage:  "start llama.cpp observer agent",
				Action: agent.Action,
			},
			{
				Name:   "balancer",
				Usage:  "start load balancer reverse proxy and Paddler metadata server",
				Action: balancer.Action,
				Flags: []cli.Flag{
					&cli.StringFlag{
						Name:        "management-host",
						Value:       "127.0.0.1",
						Destination: &balancer.ManagementServerConfiguration.HttpAddress.Host,
					},
					&cli.UintFlag{
						Name:        "management-port",
						Value:       8088,
						Destination: &balancer.ManagementServerConfiguration.HttpAddress.Port,
					},
					&cli.StringFlag{
						Name:        "management-scheme",
						Value:       "http",
						Destination: &balancer.ManagementServerConfiguration.HttpAddress.Scheme,
					},
					&cli.StringFlag{
						Name:        "reverseproxy-host",
						Value:       "127.0.0.1",
						Destination: &balancer.ReverseProxyConfiguration.HttpAddress.Host,
					},
					&cli.UintFlag{
						Name:        "reverseproxy-port",
						Value:       8087,
						Destination: &balancer.ReverseProxyConfiguration.HttpAddress.Port,
					},
					&cli.StringFlag{
						Name:        "reverseproxy-scheme",
						Value:       "http",
						Destination: &balancer.ReverseProxyConfiguration.HttpAddress.Scheme,
					},
				},
			},
		},
	}

	err := app.Run(os.Args)

	if err != nil {
		panic(err)
	}
}
