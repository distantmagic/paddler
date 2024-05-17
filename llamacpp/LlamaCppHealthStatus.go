package llamacpp

type LlamaCppHealthStatus struct {
	Error           error                    `json:"-"`
	ErrorMessage    string                   `json:"error_message"`
	Status          LlamaCppHealthStatusCode `json:"status"`
	SlotsIdle       uint                     `json:"slots_idle"`
	SlotsProcessing uint                     `json:"slots_processing"`
}
