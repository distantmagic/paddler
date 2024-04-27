package llamacpp

// {"status":"ok","slots_idle":8,"slots_processing":0}

type LlamaCppHealthStatus struct {
	Status LlamaCppHealthStatusCode `json:"status"`
	SlotsIdle uint `json:"slots_idle"`
	SlotsProcessing uint `json:"slots_processing"`
	Error error `json:"-"`
}
