package llamacpp

import (
	"context"
	"net/http"
	"testing"

	"github.com/distantmagic/paddler/netcfg"
	"github.com/stretchr/testify/assert"
)

var llamaCppClient *LlamaCppClient = &LlamaCppClient{
	HttpClient: http.DefaultClient,
	LlamaCppConfiguration: &LlamaCppConfiguration{
		HttpAddress: &netcfg.HttpAddressConfiguration{
			Host:   "127.0.0.1",
			Port:   8081,
			Scheme: "http",
		},
	},
}

func TestHealthIsObtained(t *testing.T) {
	responseChannel := make(chan LlamaCppHealthStatus)

	defer close(responseChannel)

	go llamaCppClient.GetHealth(
		context.Background(),
		responseChannel,
	)

	healthStatus := <-responseChannel

	assert.Nil(t, healthStatus.Error)
}

func TestCompletionsAreGenerated(t *testing.T) {
	responseChannel := make(chan LlamaCppCompletionToken)

	go llamaCppClient.GenerateCompletion(
		context.Background(),
		responseChannel,
		LlamaCppCompletionRequest{
			NPredict: 3,
			Prompt:   "Who are you?",
			Stream:   true,
		},
	)

	var generatedTokens int = 0

	for token := range responseChannel {
		if token.Error != nil {
			t.Fatal(token.Error)
		} else {
			generatedTokens += 1
		}
	}

	// 3 tokens + 1 summary token
	assert.Equal(t, 4, generatedTokens)
}

func TestSlotsAreObtained(t *testing.T) {
	// the test assumes llama.cpp instance running with 4 available slots
	// all of them idle

	responseChannel := make(chan LlamaCppSlotStatus)

	go llamaCppClient.GetSlots(
		context.Background(),
		responseChannel,
	)

	var totalStatuses int

	for slotStatus := range responseChannel {
		totalStatuses += 1

		assert.Nil(t, slotStatus.Error)
		assert.Equal(t, slotStatus.State, 0)
	}

	assert.Equal(t, totalStatuses, 4)
}

func TestSlotsAggregatedStatusIsbtained(t *testing.T) {
	responseChannel := make(chan LlamaCppSlotsAggregatedStatus)

	go llamaCppClient.GetSlotsAggregatedStatus(
		context.Background(),
		responseChannel,
	)

	slotsAggregatedStatus := <-responseChannel

	assert.Nil(t, slotsAggregatedStatus.Error)
	assert.Equal(t, slotsAggregatedStatus.SlotsIdle, 4)
	assert.Equal(t, slotsAggregatedStatus.SlotsProcessing, 0)
}

func TestJsonSchemaConstrainedCompletionsAreGenerated(t *testing.T) {
	responseChannel := make(chan LlamaCppCompletionToken)

	go llamaCppClient.GenerateCompletion(
		context.Background(),
		responseChannel,
		LlamaCppCompletionRequest{
			JsonSchema: map[string]any{
				"type": "object",
				"properties": map[string]any{
					"hello": map[string]string{
						"type": "string",
					},
				},
			},
			NPredict: 100,
			Prompt:   "Say 'world' as a hello!",
			Stream:   true,
		},
	)

	acc := ""

	for token := range responseChannel {
		if token.Error != nil {
			t.Fatal(token.Error)
		} else {
			acc += token.Content
		}
	}

	assert.Equal(t, "{ \"hello\": \"world\" } ", acc)
}
