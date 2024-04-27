package llamacpp

import (
	"bufio"
	"bytes"
	"encoding/json"
	"io"
	"net/http"
)

type LlamaCppClient struct {
	httpClient http.Client
}

func (llamaCppClient LlamaCppClient) GetHealth(
	responseChannel chan string,
	errorChannel chan error,
) {}

func (llamaCppClient LlamaCppClient) GenerateCompletion(
	responseChannel chan LlamaCppCompletionToken,
) {
	defer close(responseChannel)

	body := []byte(`{
		"n_predict": 100,
		"prompt": "Hello",
		"stream": true
	}`)

	request, err := http.NewRequest(
		"POST",
		"http://127.0.0.1:8081/completion",
		bytes.NewBuffer(body),
	)

	if err != nil {
		responseChannel <- LlamaCppCompletionToken{
			Error: err,
		}

		return
	}

	response, err := llamaCppClient.httpClient.Do(request)

	if err != nil {
		responseChannel <- LlamaCppCompletionToken{
			Error: err,
		}

		return
	}

	defer response.Body.Close()

	reader := bufio.NewReader(response.Body)

	for {
		line, err := reader.ReadBytes('\n')

		if err != nil && err != io.EOF {
			responseChannel <- LlamaCppCompletionToken{
				Error: err,
			}

			continue
		}

		var llamaCppCompletionToken LlamaCppCompletionToken

		trimmedLine := bytes.TrimPrefix(line, []byte("data: "))

		if len(trimmedLine) < 2 {
			continue
		}

		err = json.Unmarshal(trimmedLine, &llamaCppCompletionToken)

		if err != nil {
			responseChannel <- LlamaCppCompletionToken{
				Error: err,
			}

			continue
		}

		responseChannel <- llamaCppCompletionToken

		if llamaCppCompletionToken.IsLast {
			break
		}
	}
}
