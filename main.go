package main

import (
	"context"
	"os"

	"github.com/distantmagic/paddler/agent"
	"github.com/distantmagic/paddler/cmd"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/loadbalancer"
	"github.com/distantmagic/paddler/management"
	"github.com/distantmagic/paddler/netcfg"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
	"github.com/urfave/cli/v2"
)

const (
	DefaultManagementHost   = "127.0.0.1"
	DefaultManagementPort   = 8085
	DefaultManagementScheme = "http"
)

func main() {
	backgroundContext := context.Background()
	logger := hclog.New(&hclog.LoggerOptions{
		Name:  "paddler",
		Level: hclog.Debug,
	})
	hostResolver := &netcfg.HostResolver{
		Logger: logger,
	}

	agent := &cmd.Agent{
		AgentConfiguration: &agent.AgentConfiguration{},
		ExternalLlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{},
		},
		LocalLlamaCppConfiguration: &llamacpp.LlamaCppConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{},
		},
		Logger: logger.Named("Agent"),
		ManagementServerConfiguration: &management.ManagementServerConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{},
		},
	}

	balancer := &cmd.Balancer{
		Logger: logger.Named("Balancer"),
		ManagementServerConfiguration: &management.ManagementServerConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{},
		},
		ReverseProxyConfiguration: &reverseproxy.ReverseProxyConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{},
		},
		StatsdConfiguration: &loadbalancer.StatsdConfiguration{
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
				Before: func(cCtx *cli.Context) error {
					var host string
					var err error

					host, err = hostResolver.ResolveHost(backgroundContext, agent.ExternalLlamaCppConfiguration.HttpAddress.Host)

					if err != nil {
						return err
					}

					agent.ExternalLlamaCppConfiguration.HttpAddress.Host = host

					host, err = hostResolver.ResolveHost(backgroundContext, agent.LocalLlamaCppConfiguration.HttpAddress.Host)

					if err != nil {
						return err
					}

					agent.LocalLlamaCppConfiguration.HttpAddress.Host = host

					host, err = hostResolver.ResolveHost(backgroundContext, agent.ManagementServerConfiguration.HttpAddress.Host)

					if err != nil {
						return err
					}

					agent.ManagementServerConfiguration.HttpAddress.Host = host

					return nil
				},
				Flags: []cli.Flag{
					&cli.UintFlag{
						Name:        "agent-reporting-interval-miliseconds",
						Value:       1000,
						Destination: &agent.AgentConfiguration.ReportingIntervalMiliseconds,
					},
					&cli.StringFlag{
						Name:        "external-llamacpp-host",
						Value:       "127.0.0.1",
						Destination: &agent.ExternalLlamaCppConfiguration.HttpAddress.Host,
					},
					&cli.UintFlag{
						Name:        "external-llamacpp-port",
						Value:       8088,
						Destination: &agent.ExternalLlamaCppConfiguration.HttpAddress.Port,
					},
					&cli.StringFlag{
						Name:        "external-llamacpp-scheme",
						Value:       "http",
						Destination: &agent.ExternalLlamaCppConfiguration.HttpAddress.Scheme,
					},
					&cli.StringFlag{
						Name:        "local-llamacpp-host",
						Value:       "127.0.0.1",
						Destination: &agent.LocalLlamaCppConfiguration.HttpAddress.Host,
					},
					&cli.UintFlag{
						Name:        "local-llamacpp-port",
						Value:       8088,
						Destination: &agent.LocalLlamaCppConfiguration.HttpAddress.Port,
					},
					&cli.StringFlag{
						Name:        "local-llamacpp-scheme",
						Value:       "http",
						Destination: &agent.LocalLlamaCppConfiguration.HttpAddress.Scheme,
					},
					&cli.StringFlag{
						Name:        "management-host",
						Value:       DefaultManagementHost,
						Destination: &agent.ManagementServerConfiguration.HttpAddress.Host,
					},
					&cli.UintFlag{
						Name:        "management-port",
						Value:       DefaultManagementPort,
						Destination: &agent.ManagementServerConfiguration.HttpAddress.Port,
					},
					&cli.StringFlag{
						Name:        "management-scheme",
						Value:       DefaultManagementScheme,
						Destination: &agent.ManagementServerConfiguration.HttpAddress.Scheme,
					},
				},
			},
			{
				Name:   "balancer",
				Usage:  "start load balancer reverse proxy and Paddler metadata server",
				Action: balancer.Action,
				Before: func(cCtx *cli.Context) error {
					var host string
					var err error

					host, err = hostResolver.ResolveHost(backgroundContext, balancer.ManagementServerConfiguration.HttpAddress.Host)

					if err != nil {
						return err
					}

					balancer.ManagementServerConfiguration.HttpAddress.Host = host

					host, err = hostResolver.ResolveHost(backgroundContext, balancer.ReverseProxyConfiguration.HttpAddress.Host)

					if err != nil {
						return err
					}

					balancer.ReverseProxyConfiguration.HttpAddress.Host = host

					host, err = hostResolver.ResolveHost(backgroundContext, balancer.StatsdConfiguration.HttpAddress.Host)

					if err != nil {
						return err
					}

					balancer.StatsdConfiguration.HttpAddress.Host = host

					return nil
				},
				Flags: []cli.Flag{
					&cli.BoolFlag{
						Name:        "management-dashboard-enable",
						Value:       false,
						Destination: &balancer.ManagementServerConfiguration.EnableDashboard,
					},
					&cli.StringFlag{
						Name:        "management-host",
						Value:       DefaultManagementHost,
						Destination: &balancer.ManagementServerConfiguration.HttpAddress.Host,
					},
					&cli.UintFlag{
						Name:        "management-port",
						Value:       DefaultManagementPort,
						Destination: &balancer.ManagementServerConfiguration.HttpAddress.Port,
					},
					&cli.StringFlag{
						Name:        "management-scheme",
						Value:       DefaultManagementScheme,
						Destination: &balancer.ManagementServerConfiguration.HttpAddress.Scheme,
					},
					&cli.StringFlag{
						Name:        "reverseproxy-host",
						Value:       "127.0.0.1",
						Destination: &balancer.ReverseProxyConfiguration.HttpAddress.Host,
					},
					&cli.UintFlag{
						Name:        "reverseproxy-port",
						Value:       8086,
						Destination: &balancer.ReverseProxyConfiguration.HttpAddress.Port,
					},
					&cli.StringFlag{
						Name:        "reverseproxy-scheme",
						Value:       "http",
						Destination: &balancer.ReverseProxyConfiguration.HttpAddress.Scheme,
					},
					&cli.BoolFlag{
						Name:        "statsd-enable",
						Value:       false,
						Destination: &balancer.StatsdConfiguration.EnableStatsdReporter,
					},
					&cli.StringFlag{
						Name:        "statsd-host",
						Value:       "127.0.0.1",
						Destination: &balancer.StatsdConfiguration.HttpAddress.Host,
					},
					&cli.UintFlag{
						Name:        "statsd-port",
						Value:       8125,
						Destination: &balancer.StatsdConfiguration.HttpAddress.Port,
					},
					&cli.StringFlag{
						Name:        "statsd-scheme",
						Value:       "http",
						Destination: &balancer.StatsdConfiguration.HttpAddress.Scheme,
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
