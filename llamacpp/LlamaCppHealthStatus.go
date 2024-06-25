package llamacpp

type LlamaCppHealthStatus struct {
	Error           error                    `json:"-"`
	ErrorMessage    string                   `json:"error_message,omitempty"`
	Status          LlamaCppHealthStatusCode `json:"status"`
	SlotsIdle       int                      `json:"slots_idle"`
	SlotsProcessing int                      `json:"slots_processing"`
}

func (self *LlamaCppHealthStatus) IsOk() bool {
	return self.Status == Ok
}

func (self *LlamaCppHealthStatus) CopyFrom(other *LlamaCppHealthStatus) {
	self.Error = other.Error
	self.ErrorMessage = other.ErrorMessage
	self.SlotsIdle = other.SlotsIdle
	self.SlotsProcessing = other.SlotsProcessing
	self.Status = other.Status
}
