package llamacpp

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

var llamaCppClient *LlamaCppClient = &LlamaCppClient{
	LlamaCppConfiguration: LlamaCppConfiguration{
		Host:   "127.0.0.1",
		Port:   8081,
		Scheme: "http",
	},
}

func TestHealthIsObtained(t *testing.T) {
	responseChannel := make(chan LlamaCppHealthStatus)

	go llamaCppClient.GetHealth(responseChannel)

	healthStatus := <-responseChannel

	assert.Nil(t, healthStatus.Error)
	assert.Greater(t, healthStatus.SlotsIdle, uint(0))
	assert.GreaterOrEqual(t, healthStatus.SlotsProcessing, uint(0))
}

func TestCompletionsAreGenerated(t *testing.T) {
	responseChannel := make(chan LlamaCppCompletionToken)

	go llamaCppClient.GenerateCompletion(
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
