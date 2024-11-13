package llamacpp

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
)

const (
	CompletionDataPrefix = "data: "
)

var (
	ErrorNon200Response = errors.New("Non-200 response from llama.cpp")
)

type LlamaCppClient struct {
	HttpClient            *http.Client
	LlamaCppConfiguration *LlamaCppConfiguration
}

func (self *LlamaCppClient) GetHealth(
	ctx context.Context,
	responseChannel chan<- LlamaCppHealthStatus,
) {
	request, err := http.NewRequestWithContext(
		ctx,
		"GET",
		self.LlamaCppConfiguration.HttpAddress.BuildUrlWithPath("health").String(),
		nil,
	)

	if err != nil {
		responseChannel <- LlamaCppHealthStatus{
			Error:        err,
			ErrorMessage: err.Error(),
			Status:       Error,
		}

		return
	}

	response, err := self.HttpClient.Do(request)

	if err != nil {
		responseChannel <- LlamaCppHealthStatus{
			Error:        err,
			ErrorMessage: err.Error(),
			Status:       Error,
		}

		return
	}

	defer response.Body.Close()

	if http.StatusOK != response.StatusCode {
		responseChannel <- LlamaCppHealthStatus{
			Error:        ErrorNon200Response,
			ErrorMessage: ErrorNon200Response.Error(),
			Status:       Error,
		}

		return
	}

	var llamaCppHealthStatus LlamaCppHealthStatus

	err = json.NewDecoder(response.Body).Decode(&llamaCppHealthStatus)

	if err != nil {
		responseChannel <- LlamaCppHealthStatus{
			Error:        err,
			ErrorMessage: err.Error(),
			Status:       Error,
		}

		return
	}

	responseChannel <- llamaCppHealthStatus
}

func (self *LlamaCppClient) GetSlots(
	ctx context.Context,
	responseChannel chan<- LlamaCppSlotStatus,
) {
	defer close(responseChannel)

	request, err := http.NewRequestWithContext(
		ctx,
		"GET",
		self.LlamaCppConfiguration.HttpAddress.BuildUrlWithPath("slots").String(),
		nil,
	)

	if err != nil {
		responseChannel <- LlamaCppSlotStatus{
			Error:        err,
			ErrorMessage: err.Error(),
			State:        0,
		}

		return
	}

	if self.LlamaCppConfiguration.ApiKey != "" {
		request.Header.Set("Authorization", "Bearer "+self.LlamaCppConfiguration.ApiKey)
	}

	response, err := self.HttpClient.Do(request)

	if err != nil {
		responseChannel <- LlamaCppSlotStatus{
			Error:        err,
			ErrorMessage: err.Error(),
			State:        0,
		}

		return
	}

	defer response.Body.Close()

	if http.StatusOK != response.StatusCode {
		responseChannel <- LlamaCppSlotStatus{
			Error:        ErrorNon200Response,
			ErrorMessage: ErrorNon200Response.Error(),
			State:        0,
		}

		return
	}

	var llamaCppSlotStatuses []LlamaCppSlotStatus

	err = json.NewDecoder(response.Body).Decode(&llamaCppSlotStatuses)

	if err != nil {
		responseChannel <- LlamaCppSlotStatus{
			Error:        err,
			ErrorMessage: err.Error(),
			State:        0,
		}

		return
	}

	for _, llamaCppSlotStatus := range llamaCppSlotStatuses {
		responseChannel <- llamaCppSlotStatus
	}
}

func (self *LlamaCppClient) GetSlotsAggregatedStatus(
	ctx context.Context,
	responseChannel chan<- LlamaCppSlotsAggregatedStatus,
) {
	var llamaCppSlotsAggregatedStatus LlamaCppSlotsAggregatedStatus

	slotsChannel := make(chan LlamaCppSlotStatus)

	go self.GetSlots(ctx, slotsChannel)

	for slotStatus := range slotsChannel {
		if slotStatus.Error != nil {
			responseChannel <- LlamaCppSlotsAggregatedStatus{
				Error:        slotStatus.Error,
				ErrorMessage: slotStatus.ErrorMessage,
				Status:       Error,
			}

			return
		}

		if slotStatus.IsProcessing() {
			llamaCppSlotsAggregatedStatus.SlotsProcessing += 1
		} else {
			llamaCppSlotsAggregatedStatus.SlotsIdle += 1
		}
	}

	responseChannel <- llamaCppSlotsAggregatedStatus
}
