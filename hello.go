package main

import (
	"log"

	"github.com/distantmagic/paddler/llamacpp"
)

func main() {
	llamaCppClient := &llamacpp.LlamaCppClient{}

	responseChannel := make(chan llamacpp.LlamaCppCompletionToken)

	go llamaCppClient.GenerateCompletion(responseChannel)

	for token := range responseChannel {
		if token.Error != nil {
			log.Printf("token_error: %+v\n", token)
		} else {
			log.Printf("token: %+v\n", token.Content)
		}
	}
}
