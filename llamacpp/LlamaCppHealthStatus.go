package llamacpp

type LlamaCppHealthStatus struct {
	Status          LlamaCppHealthStatusCode `json:"status"`
	SlotsIdle       uint                     `json:"slots_idle"`
	SlotsProcessing uint                     `json:"slots_processing"`
	Error           error                    `json:"-"`
}
