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
	assert.Greater(t, healthStatus.SlotsIdle, uint(0))
	assert.GreaterOrEqual(t, healthStatus.SlotsProcessing, uint(0))
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
