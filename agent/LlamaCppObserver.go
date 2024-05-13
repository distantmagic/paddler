package agent

import (
	"time"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/management"
	"github.com/hashicorp/go-hclog"
)

type LlamaCppObserver struct {
	AgentConfiguration *AgentConfiguration
	LlamaCppClient     *llamacpp.LlamaCppClient
	Logger             hclog.Logger
	ManagementClient   *management.Client
}

func (self *LlamaCppObserver) ObserveAndReport(
	serverEventsChannel chan goroutine.ResultMessage,
) {
	llamaCppHealthStatusChannel := make(chan llamacpp.LlamaCppHealthStatus)

	defer close(llamaCppHealthStatusChannel)

	ticker := time.NewTicker(self.AgentConfiguration.GetReportingIntervalDuration())

	go self.RunTickerInterval(llamaCppHealthStatusChannel, ticker)

	for llamaCppHealthStatus := range llamaCppHealthStatusChannel {
		go self.ManagementClient.ReportLlamaCppHealthStatus(
			serverEventsChannel,
			self.LlamaCppClient.LlamaCppConfiguration,
			&llamaCppHealthStatus,
		)
	}
}

func (self *LlamaCppObserver) RunTickerInterval(
	llamaCppHealthStatusChannel chan llamacpp.LlamaCppHealthStatus,
	ticker *time.Ticker,
) {
	for range ticker.C {
		go self.LlamaCppClient.GetHealth(llamaCppHealthStatusChannel)
	}
}
