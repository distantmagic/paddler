package llamacpp

type LlamaCppSlotsAggregatedStatus struct {
	Error           error                    `json:"-"`
	ErrorMessage    string                   `json:"error_message,omitempty"`
	Status          LlamaCppHealthStatusCode `json:"status"`
	SlotsIdle       int                      `json:"slots_idle"`
	SlotsProcessing int                      `json:"slots_processing"`
}

func (self *LlamaCppSlotsAggregatedStatus) CopyFrom(other *LlamaCppSlotsAggregatedStatus) {
	self.Error = other.Error
	self.ErrorMessage = other.ErrorMessage
	self.Status = other.Status
	self.SlotsIdle = other.SlotsIdle
	self.SlotsProcessing = other.SlotsProcessing
}
