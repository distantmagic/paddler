package llamacpp

type LlamaCppSlotsAggregatedStatus struct {
	Error           error  `json:"-"`
	ErrorMessage    string `json:"error_message,omitempty"`
	SlotsIdle       int    `json:"slots_idle"`
	SlotsProcessing int    `json:"slots_processing"`
}
