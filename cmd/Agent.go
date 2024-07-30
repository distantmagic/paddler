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
	StatusServerConfiguration     *agent.StatusServerConfiguration
}

func (self *Agent) Action(cliContext *cli.Context) error {
	serverEventsChannel := make(chan goroutine.ResultMessage)

	defer close(serverEventsChannel)

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

	statusServer := &agent.StatusServer{
		Logger: self.Logger.Named("StatusServer"),
		RespondToHealth: &agent.RespondToHealth{
			ServerEventsChannel: serverEventsChannel,
		},
		StatusServerConfiguration: self.StatusServerConfiguration,
	}

	go llamaCppObserver.ObserveAndReport(serverEventsChannel)
	go statusServer.Serve(serverEventsChannel)

	for serverEvent := range serverEventsChannel {
		self.Logger.Info(
			"server event",
			"event", serverEvent,
		)
	}

	return nil
}
