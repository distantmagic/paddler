package llamacpp

type LlamaCppHealthStatus struct {
	Error           error                    `json:"-"`
	ErrorMessage    string                   `json:"error_message,omitempty"`
	Status          LlamaCppHealthStatusCode `json:"status"`
	SlotsIdle       uint                     `json:"slots_idle"`
	SlotsProcessing uint                     `json:"slots_processing"`
}

func (self *LlamaCppHealthStatus) IsOk() bool {
	return self.Status == Ok
}
