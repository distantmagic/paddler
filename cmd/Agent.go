package cmd

import (
	"net/http"

	"github.com/distantmagic/paddler/agent"
	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/management"
	"github.com/hashicorp/go-hclog"
	"github.com/urfave/cli/v2"
)

type Agent struct {
	AgentConfiguration            *agent.AgentConfiguration
	ExternalLlamaCppConfiguration *llamacpp.LlamaCppConfiguration
	LocalLlamaCppConfiguration    *llamacpp.LlamaCppConfiguration
	Logger                        hclog.Logger
	ManagementServerConfiguration *management.ManagementServerConfiguration
}

func (self *Agent) Action(cliContext *cli.Context) error {
	serverEventsChannel := make(chan goroutine.ResultMessage)

	llamaCppObserver := &agent.LlamaCppObserver{
		AgentConfiguration:            self.AgentConfiguration,
		ExternalLlamaCppConfiguration: self.ExternalLlamaCppConfiguration,
		LlamaCppClient: &llamacpp.LlamaCppClient{
			HttpClient:            http.DefaultClient,
			LlamaCppConfiguration: self.LocalLlamaCppConfiguration,
		},
		Logger: self.Logger.Named("LlamaCppObserver"),
		ManagementClient: &management.Client{
			HttpClient:                    http.DefaultClient,
			ManagementServerConfiguration: self.ManagementServerConfiguration,
		},
	}

	go llamaCppObserver.ObserveAndReport(serverEventsChannel)

	for serverEvent := range serverEventsChannel {
		self.Logger.Info(
			"server event",
			"event", serverEvent,
		)
	}

	return nil
}
