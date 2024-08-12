package management

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/distantmagic/paddler/goroutine"
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/loadbalancer"
)

type Client struct {
	HttpClient                    *http.Client
	ManagementServerConfiguration *ManagementServerConfiguration
}

func (self *Client) ReportLlamaCppHealthStatus(
	ctx context.Context,
	serverEventsChannel chan<- goroutine.ResultMessage,
	llamaCppConfiguration *llamacpp.LlamaCppConfiguration,
	llamaCppHealthStatus *llamacpp.LlamaCppHealthStatus,
	llamaCppTargetId string,
	llamaCppTargetName string,
) {
	jsonData, err := json.Marshal(&RegisterTargetRequest{
		LlamaCppHealthStatus: llamaCppHealthStatus,
		LlamaCppTargetConfiguration: &loadbalancer.LlamaCppTargetConfiguration{
			Id:                    llamaCppTargetId,
			Name:                  llamaCppTargetName,
			LlamaCppConfiguration: llamaCppConfiguration,
		},
	})

	fmt.Println(string(jsonData))

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to marshal JSON data",
			Error:   err,
		}

		return
	}

	request, err := http.NewRequestWithContext(
		ctx,
		"POST",
		self.
			ManagementServerConfiguration.
			HttpAddress.
			BuildUrlWithPath("/register/target").
			String(),
		bytes.NewBuffer(jsonData),
	)

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to create HTTP request",
			Error:   err,
		}

		return
	}

	request.Header.Set("Content-Type", "application/json")

	response, err := self.HttpClient.Do(request)

	if err != nil {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "failed to send HTTP request",
			Error:   err,
		}

		return
	}

	if response.StatusCode != http.StatusOK {
		serverEventsChannel <- goroutine.ResultMessage{
			Comment: "unexpected HTTP status code",
			Error:   err,
		}

		return
	}
}
