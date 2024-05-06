package loadbalancer

import "github.com/distantmagic/paddler/llamacpp"

type LlamaCppTarget struct {
	LlamaCppClient       *llamacpp.LlamaCppClient
	LlamaCppHealthStatus *llamacpp.LlamaCppHealthStatus
}
