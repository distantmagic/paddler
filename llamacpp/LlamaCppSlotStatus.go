package llamacpp

type LlamaCppSlotStatus struct {
	Error        error  `json:"-"`
	ErrorMessage string `json:"error_message,omitempty"`
	IsProcessing  bool  `json:"is_processing"`
}
