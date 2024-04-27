package llamacpp

type LlamaCppCompletionRequest struct {
	NPredict int    `json:"n_predict"`
	Prompt   string `json:"prompt"`
	Stream   bool   `json:"stream"`
}
