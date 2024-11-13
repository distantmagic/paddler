package llamacpp

type LlamaCppHealthStatus struct {
	Error        error                    `json:"-"`
	ErrorMessage string                   `json:"error_message,omitempty"`
	Status       LlamaCppHealthStatusCode `json:"status"`
}
