package llamacpp

type LlamaCppHealthStatus struct {
	Error           error                    `json:"-"`
	ErrorMessage    string                   `json:"error_message,omitempty"`
	Status          LlamaCppHealthStatusCode `json:"status"`
	SlotsIdle       int                      `json:"slots_idle"`
	SlotsProcessing int                      `json:"slots_processing"`
}
