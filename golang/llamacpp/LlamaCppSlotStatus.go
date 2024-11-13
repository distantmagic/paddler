package llamacpp

type LlamaCppSlotStatus struct {
	Error        error  `json:"-"`
	ErrorMessage string `json:"error_message,omitempty"`
	State        int    `json:"state"`
}

func (self *LlamaCppSlotStatus) IsProcessing() bool {
	return 1 == self.State
}
