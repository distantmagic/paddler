package agent

import (
	"context"
	"time"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/management"
	"github.com/google/uuid"
	"github.com/hashicorp/go-hclog"
)

type LlamaCppObserver struct {
	AgentConfiguration            *AgentConfiguration
	ExternalLlamaCppConfiguration *llamacpp.LlamaCppConfiguration
	LlamaCppClient                *llamacpp.LlamaCppClient
	Logger                        hclog.Logger
	ManagementClient              *management.Client
}

func (self *LlamaCppObserver) ObserveAndReport(
	serverEventsChannel chan<- goroutine.ResultMessage,
) {
	self.Logger.Debug(
		"observing",
		"host", self.LlamaCppClient.LlamaCppConfiguration.HttpAddress.GetHostWithPort(),
	)

	agentRuntimeId, err := uuid.NewV7()

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to generate agent uuid",
			Error:   err,
		}

		return
	}

	agentRuntimeIdString := agentRuntimeId.String()

	llamaCppSlotsAggregateStatusChannel := make(chan llamacpp.LlamaCppSlotsAggregatedStatus)

	defer close(llamaCppSlotsAggregateStatusChannel)

	ticker := time.NewTicker(self.AgentConfiguration.GetReportingIntervalDuration())

	go self.RunTickerInterval(llamaCppSlotsAggregateStatusChannel, ticker)

	for llamaCppSlotsAggregatedStatus := range llamaCppSlotsAggregateStatusChannel {
		ctx, cancel := context.WithTimeout(
			context.Background(),
			self.AgentConfiguration.GetReportingIntervalDuration(),
		)

		self.ManagementClient.ReportLlamaCppSlotsAggregatedStatus(
			ctx,
			serverEventsChannel,
			self.ExternalLlamaCppConfiguration,
			&llamaCppSlotsAggregatedStatus,
			agentRuntimeIdString,
			self.AgentConfiguration.Name,
		)

		cancel()
	}
}

func (self *LlamaCppObserver) RunTickerInterval(
	llamaCppSlotsAggregateStatusChannel chan llamacpp.LlamaCppSlotsAggregatedStatus,
	ticker *time.Ticker,
) {
	for range ticker.C {
		ctx, cancel := context.WithTimeout(
			context.Background(),
			self.AgentConfiguration.GetReportingIntervalDuration(),
		)

		self.LlamaCppClient.GetSlotsAggregatedStatus(
			ctx,
			llamaCppSlotsAggregateStatusChannel,
		)

		cancel()
	}
}
